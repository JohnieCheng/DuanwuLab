use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::context::error::GlobalErrorContext;
use crate::context::store::{
    AppStore, CompressResult, CompressTaskMsg, PickedFile, ResultKey, SystemMsg,
};
use crate::utils::safe_invoke::safe_invoke;

/// Initialize global state, persistence, and background workers.
///
/// Must be called once in the root layout component. Creates:
/// - `Signal<AppStore>` — in-memory state, injected via context
/// - System coroutine — async disk persistence
/// - Compress coroutine — batched image IPC processing
pub fn use_init_app_state() {
    let error_ctx = use_context::<GlobalErrorContext>();

    // 1. Create global signal store and inject into context
    let mut app_store = use_context_provider(|| Signal::new(AppStore::default()));

    // 2. Restore persisted store from disk on cold start
    use_effect(move || {
        let ctx = error_ctx;
        spawn(async move {
            if let Some(loaded) =
                safe_invoke::<AppStore, _>("load_store", serde_json::json!({}), ctx).await
            {
                *app_store.write() = loaded;
            }
        });
    });

    // 3. System coroutine — disk persistence
    let sys_worker = use_coroutine(move |mut rx: UnboundedReceiver<SystemMsg>| async move {
        while let Some(msg) = rx.next().await {
            let SystemMsg::SaveToDisk = msg;
            let store = app_store.read().clone();
            let _ = safe_invoke::<serde_json::Value, _>(
                "save_store",
                serde_json::json!({"store": store}),
                error_ctx,
            )
            .await;
        }
    });

    // 4. Compress coroutine — heavy IPC tasks only
    let compress_worker = use_coroutine(
        move |mut rx: UnboundedReceiver<CompressTaskMsg>| async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    CompressTaskMsg::AddImages => {
                        // Gotcha: release lock before awaiting IPC (never hold Signal write across .await)
                        if let Some(picked) = safe_invoke::<Vec<PickedFile>, _>(
                            "pick_image_files",
                            serde_json::json!({}),
                            error_ctx,
                        )
                        .await
                        {
                            let mut s = app_store.write();
                            let existing: Vec<_> =
                                s.compressor.files.iter().map(|f| f.path.clone()).collect();
                            for pf in picked {
                                if !existing.contains(&pf.path) {
                                    s.compressor.files.push(pf);
                                }
                            }
                        }
                        sys_worker.send(SystemMsg::SaveToDisk);
                    }

                    CompressTaskMsg::CompressAll => {
                        // Snapshot params, release lock before IPC
                        let (files, quality, mode, output_dir) = {
                            let s = app_store.read();
                            let pending: Vec<PickedFile> = s
                                .compressor
                                .files
                                .iter()
                                .filter(|f| {
                                    let key = ResultKey(f.path.clone(), s.compressor.quality);
                                    !s.compressor.results.contains_key(&key)
                                        && !s.compressor.compressing_paths.contains(&f.path)
                                })
                                .cloned()
                                .collect();
                            (
                                pending,
                                s.compressor.quality,
                                s.compressor.compress_mode.clone(),
                                s.compressor.output_dir.clone(),
                            )
                        };

                        // Mark all as compressing
                        {
                            let mut s = app_store.write();
                            for f in &files {
                                s.compressor.compressing_paths.insert(f.path.clone());
                            }
                        }

                        // Controlled concurrency: max 4 parallel compressions
                        let sw = sys_worker;
                        let ctx = error_ctx;
                        let q = quality;
                        let m = mode.clone();
                        let od = output_dir.clone();
                        spawn(async move {
                            use futures_util::StreamExt;
                            futures_util::stream::iter(&files)
                                .for_each_concurrent(4, |file| {
                                    let path = file.path.clone();
                                    let m = m.clone();
                                    let od = od.clone();
                                    async move {
                                        let res = safe_invoke::<CompressResult, _>(
                                            "compress_single_image",
                                            serde_json::json!({"path": &path, "quality": q, "mode": &m, "outputDir": &od}),
                                            ctx,
                                        ).await;

                                        let mut s = app_store.write();
                                        s.compressor.compressing_paths.remove(&path);
                                        if let Some(r) = res {
                                            s.compressor.results.insert(ResultKey(path, q), Ok(r));
                                        } else {
                                            s.compressor.results.insert(ResultKey(path, q), Err("Compression failed".to_string()));
                                        }
                                        sw.send(SystemMsg::SaveToDisk);
                                    }
                                })
                                .await;
                        });
                    }
                }
            }
        },
    );

    use_context_provider(|| compress_worker);
}

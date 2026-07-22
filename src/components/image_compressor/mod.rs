mod file_list;
mod preview;
mod toolbar;
mod types;
mod utils;

use dioxus::prelude::*;
use file_list::FileList;
use toolbar::Toolbar;

use crate::context::error::GlobalErrorContext;
use crate::context::store::{AppStore, CompressResult, CompressTaskMsg, ResultKey};
use crate::utils::safe_invoke::safe_invoke;

/// Image compression tool page.
/// Renders toolbar, file list, and preview modal.
/// All heavy IPC is delegated to the global coroutine (`CompressTaskMsg`).
#[component]
pub fn ImageCompressor() -> Element {
    let mut app_store = use_context::<Signal<AppStore>>();
    let compress_worker = use_context::<Coroutine<CompressTaskMsg>>();
    let error_ctx = use_context::<GlobalErrorContext>();

    // Shared loading state — Toolbar/FileList both need it
    let loading_preview = use_signal(|| false);

    // Reactive reads from AppStore
    let files = use_memo(move || app_store.read().compressor.files.clone());
    let results = use_memo(move || app_store.read().compressor.results.clone());
    let compressing_paths = use_memo(move || app_store.read().compressor.compressing_paths.clone());
    let quality = use_memo(move || app_store.read().compressor.quality);
    let compress_mode = use_memo(move || app_store.read().compressor.compress_mode.clone());
    let output_dir = use_memo(move || app_store.read().compressor.output_dir.clone());

    // --- Handlers ---

    let handle_mode = move |m: String| {
        app_store.write().compressor.compress_mode = m;
    };

    let handle_quality = move |e: FormEvent| {
        if let Ok(v) = e.value().parse::<u8>() {
            app_store.write().compressor.quality = v.clamp(1, 99);
        }
    };

    let handle_pick_dir = move |_: MouseEvent| {
        let ctx = error_ctx;
        spawn(async move {
            if let Some(Some(dir)) =
                safe_invoke::<Option<String>, _>("pick_output_dir", serde_json::json!({}), ctx)
                    .await
            {
                app_store.write().compressor.output_dir = Some(dir);
            }
        });
    };

    let handle_remove = move |path: String| {
        let mut s = app_store.write();
        let q = s.compressor.quality;
        s.compressor.files.retain(|f| f.path != path);
        s.compressor.results.remove(&ResultKey(path.clone(), q));
        s.compressor.compressing_paths.remove(&path);
    };

    let handle_open_dir = {
        let ctx = error_ctx;
        move |_: MouseEvent| {
            let dir = app_store.read().compressor.output_dir.clone();
            let target = dir.unwrap_or_else(|| {
                // Fallback: parent of first compressed file
                app_store
                    .read()
                    .compressor
                    .results
                    .values()
                    .find_map(|r| {
                        r.as_ref().ok().map(|r| {
                            std::path::Path::new(&r.compressed_path)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default()
                        })
                    })
                    .unwrap_or_default()
            });
            if !target.is_empty() {
                spawn(async move {
                    let _ = safe_invoke::<serde_json::Value, _>(
                        "open_dir",
                        serde_json::json!({"path": target}),
                        ctx,
                    )
                    .await;
                });
            }
        }
    };

    // Computed — memoized to avoid re-computation on unrelated state changes
    let file_count = files.read().len();
    let is_any_compressing = !compressing_paths.read().is_empty();

    let stats = use_memo(move || {
        let res_map = results.read();
        let q = *quality.read();
        let matching: Vec<&CompressResult> =
            res_map.values().filter_map(|r| r.as_ref().ok()).filter(|r| r.quality == q).collect();
        let processed_count = matching.len();
        let total_original: u64 = matching.iter().map(|r| r.original).sum();
        let total_compressed: u64 = matching.iter().map(|r| r.compressed).sum();
        let saved_pct = if total_original > total_compressed {
            (total_original - total_compressed) as f64 / (total_original as f64).max(1.0) * 100.0
        } else {
            0.0
        };
        (processed_count, total_original, total_compressed, saved_pct)
    });

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            Toolbar {
                quality: *quality.read(),
                compress_mode: compress_mode.read().clone(),
                output_dir,
                is_any_compressing,
                file_count,
                processed_count: stats.read().0,
                total_original: stats.read().1,
                total_compressed: stats.read().2,
                total_saved_pct: stats.read().3,
                on_mode: EventHandler::new(handle_mode),
                on_quality: EventHandler::new(handle_quality),
                on_pick_dir: EventHandler::new(handle_pick_dir),
                on_clear_dir: EventHandler::new(move |_| app_store.write().compressor.output_dir = None),
                on_add_images: EventHandler::new(move |_| compress_worker.send(CompressTaskMsg::AddImages)),
                on_compress_all: EventHandler::new(move |_| compress_worker.send(CompressTaskMsg::CompressAll)),
                on_clear_all: EventHandler::new(move |_| {
                    let mut s = app_store.write();
                    s.compressor.files.clear();
                    s.compressor.results.clear();
                    s.compressor.compressing_paths.clear();
                }),
                on_open_dir: EventHandler::new(handle_open_dir),
                loading: loading_preview(),
            }

            FileList {
                files,
                results,
                compressing_paths,
                quality: *quality.read(),
                loading_preview,
                on_remove: EventHandler::new(handle_remove),
                is_any_compressing,
            }
            if file_count == 0 {
                div { class: "flex flex-1 items-center justify-center",
                    div { class: "flex flex-col items-center gap-3 text-center",
                        p { class: "text-sm text-gray-400 dark:text-gray-500 max-w-xs",
                            "Click \"Add Images\" to start. Click the eye icon to preview."
                        }
                    }
                }
            }
        }
    }
}

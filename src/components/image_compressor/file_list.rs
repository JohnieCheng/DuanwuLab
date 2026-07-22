use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;

use super::types::{PreviewData, PreviewState};
use super::utils::{asset_url, format_size, saved_percent};
use crate::context::error::GlobalErrorContext;
use crate::context::store::{CompressResult, PickedFile, ResultKey};
use crate::utils::safe_invoke::safe_invoke;

/// File list with per-item actions (preview, remove)
#[component]
pub(super) fn FileList(
    files: ReadSignal<Vec<PickedFile>>,
    results: ReadSignal<HashMap<ResultKey, Result<CompressResult, String>>>,
    compressing_paths: ReadSignal<HashSet<String>>,
    quality: u8,
    on_remove: EventHandler<String>,
    is_any_compressing: bool,
    loading_preview: Signal<bool>,
) -> Element {
    let error_ctx = use_context::<GlobalErrorContext>();
    let mut preview = use_signal(|| None::<PreviewState>);
    let mut loading_path = use_signal(String::new);
    let mut active_preview_path = use_signal(String::new);
    let mut preview_cache = use_signal(HashMap::<(String, u8), PreviewData>::new);

    let show_preview = move |file_path: String| {
        let q = quality;
        active_preview_path.set(file_path.clone());

        if let Some(cached) = preview_cache.read().get(&(file_path.clone(), q)) {
            preview.set(Some(PreviewState {
                data: cached.clone(),
                original_url: asset_url(&cached.original),
                compressed_url: asset_url(&cached.compressed),
                slider_pos: 50.0,
            }));
            return;
        }

        if let Some(Ok(res)) = results.read().get(&ResultKey(file_path.clone(), q)) {
            let data = PreviewData {
                original: file_path.clone(),
                compressed: res.compressed_path.clone(),
                original_size: res.original,
                compressed_size: res.compressed,
                width: res.width,
                height: res.height,
            };
            preview_cache.write().insert((file_path.clone(), q), data.clone());
            preview.set(Some(PreviewState {
                data: data.clone(),
                original_url: asset_url(&data.original),
                compressed_url: asset_url(&data.compressed),
                slider_pos: 50.0,
            }));
            return;
        }

        loading_preview.set(true);
        loading_path.set(file_path.clone());
        preview.set(None);

        let target_path = file_path.clone();
        spawn(async move {
            if let Some(data) = safe_invoke::<PreviewData, _>(
                "preview_compress",
                serde_json::json!({"path": target_path, "quality": q}),
                error_ctx,
            )
            .await
                && *active_preview_path.read() == target_path
            {
                preview_cache.write().insert((target_path.clone(), q), data.clone());
                let comp_url = asset_url(&data.compressed);
                let orig_url = asset_url(&target_path);
                preview.set(Some(PreviewState {
                    data,
                    original_url: orig_url,
                    compressed_url: comp_url,
                    slider_pos: 50.0,
                }));
            }
            if *active_preview_path.read() == target_path {
                loading_preview.set(false);
                loading_path.set(String::new());
            }
        });
    };

    let file_count = files.read().len();

    rsx! {
        if file_count > 0 {
            div { class: "flex flex-col rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden",
                for f in files.read().iter() {
                    {
                        let path = f.path.clone();
                        let raw = results.read().get(&ResultKey(path.clone(), quality)).cloned();
                        let is_item_compressing = compressing_paths.read().contains(&path);
                        let loading = loading_preview();
                        let is_this_loading = loading && loading_path() == path;

                        rsx! {
                            div { key: "{path}",
                                class: "flex items-center justify-between px-3 py-2.5 text-xs border-b border-gray-100 dark:border-gray-800 last:border-b-0 transition-colors hover:bg-gray-50/60 dark:hover:bg-gray-800/40",
                                div { class: "flex flex-col min-w-0 pr-4",
                                    span { class: "text-gray-700 dark:text-gray-200 truncate font-medium", "{f.name}" }
                                    span { class: "text-gray-400 text-[10px] font-mono truncate",
                                        if let Some(Ok(ref res)) = raw {
                                            "{res.width}×{res.height} · {res.format}"
                                        } else if let Some(Err(ref err_msg)) = raw {
                                            "{err_msg}"
                                        } else {
                                            "{format_size(f.size)}"
                                        }
                                    }
                                }
                                div { class: "flex items-center gap-3 shrink-0",
                                    if is_this_loading {
                                        div { class: "flex items-center gap-1.5 text-blue-500 dark:text-blue-400 text-[11px]",
                                            svg { class: "w-3 h-3 animate-spin shrink-0", view_box: "0 0 24 24", fill: "none",
                                                circle { class: "opacity-25", cx: "12", cy: "12", r: "10", stroke: "currentColor", stroke_width: "4" }
                                                path { class: "opacity-75", fill: "currentColor", d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" }
                                            }
                                            span { class: "italic", "Loading..." }
                                        }
                                    } else if let Some(Ok(res)) = raw {
                                        span { class: "text-gray-400 font-mono text-[11px]", "{format_size(res.original)} → {format_size(res.compressed)}" }
                                        span { class: "text-emerald-600 dark:text-emerald-400 font-semibold font-mono text-[11px]",
                                            "-{saved_percent(res.original, res.compressed)}%"
                                        }
                                        div { class: "flex items-center gap-0.5 ml-1",
                                            button {
                                                class: "p-1.5 text-gray-400 hover:text-blue-500 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-30",
                                                disabled: is_item_compressing || loading,
                                                onclick: {
                                                    let p = path.clone();
                                                    let mut sp = show_preview;
                                                    move |_| sp(p.clone())
                                                },
                                                svg { class: "w-3.5 h-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                    path { d: "M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7z" }
                                                    circle { cx: "12", cy: "12", r: "3" }
                                                }
                                            }
                                            button {
                                                class: "p-1.5 text-gray-400 hover:text-red-500 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-30",
                                                disabled: is_item_compressing || loading,
                                                onclick: {
                                                    let p = path.clone();
                                                    move |_| on_remove.call(p.clone())
                                                },
                                                span { class: "text-xs", "✕" }
                                            }
                                        }
                                    } else if let Some(Err(_)) = raw {
                                        span { class: "text-red-500 dark:text-red-400 font-medium text-[11px]", "Failed" }
                                        button {
                                            class: "p-1.5 text-gray-400 hover:text-red-500 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-gray-800",
                                            onclick: {
                                                let p = path.clone();
                                                move |_| on_remove.call(p.clone())
                                            },
                                            span { class: "text-xs", "✕" }
                                        }
                                    } else if is_item_compressing {
                                        div { class: "flex items-center gap-1.5 text-blue-500 dark:text-blue-400 text-[11px]",
                                            svg { class: "w-3 h-3 animate-spin shrink-0", view_box: "0 0 24 24", fill: "none",
                                                circle { class: "opacity-25", cx: "12", cy: "12", r: "10", stroke: "currentColor", stroke_width: "4" }
                                                path { class: "opacity-75", fill: "currentColor", d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" }
                                            }
                                            span { class: "italic", "Compressing..." }
                                        }
                                    } else {
                                        span { class: "text-gray-400 font-mono text-[11px]", "{format_size(f.size)}" }
                                        div { class: "flex items-center gap-0.5 ml-1",
                                            button {
                                                class: "p-1.5 text-gray-400 hover:text-blue-500 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-30",
                                                disabled: is_item_compressing || loading,
                                                onclick: {
                                                    let p = path.clone();
                                                    let mut sp = show_preview;
                                                    move |_| sp(p.clone())
                                                },
                                                svg { class: "w-3.5 h-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                    path { d: "M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7z" }
                                                    circle { cx: "12", cy: "12", r: "3" }
                                                }
                                            }
                                            button {
                                                class: "p-1.5 text-gray-400 hover:text-red-500 transition-colors rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 disabled:opacity-30",
                                                disabled: is_item_compressing || loading,
                                                onclick: {
                                                    let p = path.clone();
                                                    move |_| on_remove.call(p.clone())
                                                },
                                                span { class: "text-xs", "✕" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Preview modal
            if let Some(ref ps) = *preview.read() {
                super::preview::PreviewModal {
                    state: ps.clone(),
                    loading: loading_preview(),
                    on_close: move |_| { preview.set(None); loading_preview.set(false); loading_path.set(String::new()); },
                }
            }
        }
    }
}

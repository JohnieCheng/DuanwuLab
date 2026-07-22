use dioxus::prelude::*;

use super::utils::format_size;
use crate::components::common::button::{Button, ButtonVariant};

/// Top toolbar: mode toggle, quality slider, output dir, action buttons
#[component]
pub(super) fn Toolbar(
    quality: u8,
    compress_mode: String,
    output_dir: ReadSignal<Option<String>>,
    is_any_compressing: bool,
    file_count: usize,
    processed_count: usize,
    total_original: u64,
    total_compressed: u64,
    total_saved_pct: f64,
    on_mode: EventHandler<String>,
    on_quality: EventHandler<FormEvent>,
    on_pick_dir: EventHandler<MouseEvent>,
    on_clear_dir: EventHandler<MouseEvent>,
    on_add_images: EventHandler<MouseEvent>,
    on_compress_all: EventHandler<MouseEvent>,
    on_clear_all: EventHandler<MouseEvent>,
    on_open_dir: EventHandler<MouseEvent>,
    loading: bool,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-3 select-none",
            // Row 1: mode + quality + output dir
            div { class: "flex items-center gap-4 flex-wrap",
                div { class: "flex items-center gap-1.5 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg text-xs",
                    button {
                        class: if compress_mode == "fast" {
                            "px-2.5 py-1 rounded-md font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm transition-all"
                        } else {
                            "px-2.5 py-1 rounded-md font-medium text-gray-500 hover:text-gray-900 dark:hover:text-white transition-all"
                        },
                        disabled: is_any_compressing || loading,
                        onclick: move |_| on_mode.call("fast".to_string()),
                        "Fast"
                    }
                    button {
                        class: if compress_mode == "best" {
                            "px-2.5 py-1 rounded-md font-medium bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm transition-all"
                        } else {
                            "px-2.5 py-1 rounded-md font-medium text-gray-500 hover:text-gray-900 dark:hover:text-white transition-all"
                        },
                        disabled: is_any_compressing || loading,
                        onclick: move |_| on_mode.call("best".to_string()),
                        "Best"
                    }
                }

                div { class: "flex items-center gap-2",
                    span { class: "text-[11px] text-gray-400", "Quality" }
                    input {
                        class: "w-24 h-1 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-gray-800 dark:accent-gray-200 disabled:opacity-30",
                        r#type: "range",
                        min: 1, max: 99,
                        disabled: is_any_compressing || loading,
                        value: "{quality}",
                        oninput: move |e| on_quality.call(e),
                    }
                    input {
                        class: "w-10 text-center text-[11px] font-mono text-gray-500 dark:text-gray-400 bg-transparent border border-gray-200 dark:border-gray-700 rounded px-1 py-0.5 outline-none focus:border-gray-400",
                        r#type: "number",
                        min: 1, max: 99,
                        disabled: is_any_compressing || loading,
                        value: "{quality}",
                        oninput: move |e| on_quality.call(e),
                    }
                }

                div { class: "flex items-center gap-2 ml-auto",
                    Button { label: "Select Output Dir", onclick: move |e| on_pick_dir.call(e), variant: ButtonVariant::Outline, disabled: is_any_compressing }
                    if let Some(dir) = output_dir.read().as_ref() {
                        span { class: "text-[11px] text-gray-400 truncate max-w-[320px] font-mono", "{dir}" }
                        button {
                            class: "text-gray-400 hover:text-red-500 transition-colors text-xs disabled:opacity-30",
                            disabled: is_any_compressing || loading,
                            onclick: move |e| on_clear_dir.call(e),
                            "✕"
                        }
                    } else {
                        span { class: "text-[11px] text-gray-400", "same folder" }
                    }
                }
            }

            // Row 2: actions
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    Button { label: "Add Images", onclick: move |e| on_add_images.call(e), variant: ButtonVariant::Solid, disabled: is_any_compressing }
                    if file_count > 0 {
                        Button {
                            label: if is_any_compressing { "Compressing..." }
                                   else if processed_count == file_count { "All Processed" }
                                   else { "Compress All" },
                            onclick: move |e| on_compress_all.call(e),
                            variant: ButtonVariant::Solid,
                            disabled: is_any_compressing || processed_count == file_count,
                        }
                        span { class: "text-xs text-gray-400", "{file_count} files" }
                    }
                }

                if file_count > 0 {
                    div { class: "flex items-center gap-6",
                        Button { label: "Show in Finder", onclick: move |e| on_open_dir.call(e), variant: ButtonVariant::Ghost }
                        Button { label: "Clear All", onclick: move |e| on_clear_all.call(e), variant: ButtonVariant::Danger, disabled: is_any_compressing || loading }
                    }
                }
            }

            // Progress bar
            if processed_count > 0 {
                div { class: "flex items-center justify-between px-3 py-2 bg-emerald-500/10 rounded-lg border border-emerald-500/20 text-xs",
                    span { class: "text-emerald-700 dark:text-emerald-300 font-medium", "Progress: {processed_count} of {file_count} compressed" }
                    div { class: "flex items-center gap-3 text-emerald-600 dark:text-emerald-400 font-medium",
                        span { "{format_size(total_original)} → {format_size(total_compressed)}" }
                        span { "Saved {total_saved_pct:.0}%" }
                    }
                }
            }
        }
    }
}

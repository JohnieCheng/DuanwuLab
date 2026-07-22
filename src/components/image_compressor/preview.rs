use dioxus::prelude::*;

use super::types::PreviewState;
use super::utils::format_size;

/// Preview modal with comparison slider
#[component]
pub(super) fn PreviewModal(
    state: PreviewState,
    loading: bool,
    on_close: EventHandler<MouseEvent>,
) -> Element {
    let mut dragging = use_signal(|| false);
    let mut zoom = use_signal(|| 1.0f64);
    let mut slider_pos = use_signal(|| state.slider_pos);
    let mut container_rect = use_signal(|| (0.0, 0.0));

    let pct = if state.data.original_size > 0 {
        (state.data.compressed_size as f64 / state.data.original_size as f64 * 100.0).min(100.0)
    } else {
        100.0
    };
    let saved_pct = (100.0 - pct) as u32;

    rsx! {
        div { class: "fixed inset-0 z-50 bg-black/80 flex items-center justify-center",
            onclick: move |e| on_close.call(e),
            div {
                class: "relative flex flex-col gap-3 max-w-[90vw] max-h-[90vh]",
                onclick: move |e: MouseEvent| e.stop_propagation(),
                onmousemove: move |e: MouseEvent| {
                    if dragging() {
                        let (left, width) = *container_rect.read();
                        if width > 0.0 {
                            let x = e.data().client_coordinates().x - left;
                            slider_pos.set((x / width * 100.0).clamp(0.0, 100.0));
                        }
                    }
                },
                onmouseup: move |_| dragging.set(false),
                onmouseleave: move |_| dragging.set(false),

                button {
                    class: "absolute -top-10 right-0 text-white hover:text-white/80 text-sm transition-colors",
                    onclick: move |e| on_close.call(e),
                    "✕"
                }

                div { class: "flex items-center gap-1",
                    button {
                        class: "px-2 py-0.5 text-xs text-white/60 hover:text-white bg-white/10 rounded hover:bg-white/20",
                        onclick: move |_| { let z = *zoom.read(); zoom.set((z - 0.25).max(0.5)); },
                        "−"
                    }
                    span { class: "text-xs text-white/60 w-10 text-center", "{(*zoom.read() * 100.0) as u32}%" }
                    button {
                        class: "px-2 py-0.5 text-xs text-white/60 hover:text-white bg-white/10 rounded hover:bg-white/20",
                        onclick: move |_| { let z = *zoom.read(); zoom.set((z + 0.25).min(4.0)); },
                        "+"
                    }
                    button {
                        class: "px-2 py-0.5 text-xs text-white/40 hover:text-white bg-white/10 rounded hover:bg-white/20",
                        onclick: move |_| zoom.set(1.0),
                        "1:1"
                    }
                }

                div { class: "overflow-auto custom-scrollbar", style: "max-height:70vh; max-width:80vw",
                    div { style: "width:{*zoom.read() * 100.0}%; transition: width 0.2s ease-out; margin: 0 auto;",
                        div { id: "compare-container", class: "relative overflow-hidden",
                            div { style: "position:relative; width:100%",
                                img { style: "display:block; width:100%", src: "{state.original_url}" }
                                div { class: "absolute top-2 right-2 bg-red-500/80 text-white text-[10px] px-2 py-0.5 rounded-full font-medium",
                                    "{format_size(state.data.original_size)}" }
                                div { class: "absolute top-0 left-0 w-full h-full overflow-hidden",
                                    style: "clip-path:inset(0 {100.0 - slider_pos()}% 0 0)",
                                    img { style: "display:block; width:100%", src: "{state.compressed_url}" }
                                    div { class: "absolute top-2 left-2 bg-blue-500/80 text-white text-[10px] px-2 py-0.5 rounded-full font-medium",
                                        "{format_size(state.data.compressed_size)} ({saved_pct}% smaller)" }
                                }
                            }
                            div { class: "absolute top-0 bottom-0 pointer-events-none", style: "left:{slider_pos()}%",
                                div { class: "absolute top-0 bottom-0 w-0.5 bg-white shadow-lg pointer-events-auto cursor-ew-resize",
                                    onmousedown: move |_| {
                                        dragging.set(true);
                                        if let Some(el) = web_sys::window()
                                            .and_then(|w| w.document())
                                            .and_then(|d| d.get_element_by_id("compare-container"))
                                        {
                                            let rect = el.get_bounding_client_rect();
                                            container_rect.set((rect.left(), rect.width().max(1.0)));
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                div { class: "flex justify-between text-xs text-white/60",
                    span { "Original — {format_size(state.data.original_size)}" }
                    span { "Compressed — {format_size(state.data.compressed_size)}" }
                }
                if loading {
                    div { class: "absolute inset-0 flex items-center justify-center bg-black/40 rounded-lg",
                        span { class: "text-white text-sm", "Loading preview..." }
                    }
                }
            }
        }
    }
}

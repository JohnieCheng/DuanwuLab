use dioxus::prelude::*;

/// Fixed-position notification that appears when an error signal is non-empty.
#[component]
pub fn ErrorToast(mut message: Signal<Option<String>>) -> Element {
    rsx! {
        if let Some(err) = message.cloned() {
            div { class: "fixed top-4 right-4 z-50 max-w-md",
                div { class: "flex items-center gap-2 rounded-lg border-l-2 border-red-400 bg-white \
                             pl-3 pr-2 py-2 text-xs text-gray-700 shadow \
                             dark:border-red-500 dark:bg-gray-900 dark:text-gray-300 select-none",
                    span { class: "truncate", "{err}" }
                    button {
                        class: "shrink-0 text-gray-300 hover:text-gray-500 dark:text-gray-600 dark:hover:text-gray-400 \
                                transition-colors px-1",
                        onclick: move |_| message.set(None),
                        "✕"
                    }
                }
            }
        }
    }
}

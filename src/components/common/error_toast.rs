use dioxus::prelude::*;

/// Fixed-position notification that appears when an error signal is non-empty.
#[component]
pub fn ErrorToast(mut message: Signal<Option<String>>) -> Element {
    rsx! {
        if let Some(err) = message.cloned() {
            div { class: "fixed top-4 right-4 bg-red-500 text-white p-4 rounded shadow-lg z-50",
                span { "{err}" }
                button { class: "ml-4 font-bold select-none", onclick: move |_| message.set(None), "✕" }
            }
        }
    }
}

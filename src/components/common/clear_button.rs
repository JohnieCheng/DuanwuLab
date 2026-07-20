use dioxus::document::eval;
use dioxus::prelude::*;

/// Absolute-positioned clear button for textareas.
#[component]
pub fn ClearButton(input_id: &'static str, on_clear: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "absolute top-3 right-3 rounded border border-gray-200 bg-white px-2 py-0.5 text-[11px] text-gray-400 hover:text-red-500 hover:border-red-200 dark:border-gray-700 dark:bg-gray-900 dark:hover:text-red-400 transition-colors select-none",
            onclick: move |_| {
                on_clear.call(());
                eval(&format!("var t=document.getElementById('{input_id}');if(t)t.value=''"));
            },
            "✕ Clear"
        }
    }
}

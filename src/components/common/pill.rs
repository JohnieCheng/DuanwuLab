use dioxus::prelude::*;

/// Pill-shaped toggle button.
#[component]
pub fn Pill(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let base = "rounded-full px-3 py-1 text-xs font-medium transition-colors cursor-pointer \
                select-none focus:outline-none focus:ring-2 focus:ring-offset-2 \
                focus:ring-gray-300 dark:focus:ring-gray-700";
    let state = if active {
        "bg-gray-800 text-white dark:bg-gray-200 dark:text-gray-900"
    } else {
        "bg-gray-100 text-gray-500 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 \
         dark:hover:bg-gray-700"
    };
    rsx! {
        button {
            r#type: "button",
            class: "{base} {state}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

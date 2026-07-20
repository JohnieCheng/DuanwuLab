use dioxus::prelude::*;

/// Sidebar navigation item.
#[component]
pub fn NavItem(
    label: &'static str,
    icon: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let base = "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors \
                cursor-pointer select-none";
    let state = if active {
        "bg-gray-100 text-gray-900 dark:bg-gray-800 dark:text-white"
    } else {
        "text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-400 \
         dark:hover:bg-gray-800 dark:hover:text-white"
    };
    rsx! {
        div { class: "{base} {state}", onclick: move |e| onclick.call(e),
            span { class: "inline-flex w-7 justify-center font-mono text-base", "{icon}" }
            span { class: "flex-1", "{label}" }
        }
    }
}

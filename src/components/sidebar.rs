use dioxus::prelude::*;

use crate::components::json_formatter::JsonFormatter;

/// Top-level layout: sidebar navigation + main content area.
pub fn SidebarLayout() -> Element {
    rsx! {
        div { class: "w-screen h-screen flex overflow-hidden bg-gray-50 dark:bg-gray-900 select-none",

            aside { class: "w-64 h-full flex flex-col flex-shrink-0 overflow-hidden border-r border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-950 select-none",
                div { class: "p-4 flex-shrink-0",
                    div { class: "flex items-center gap-3",
                        div { class: "h-8 w-8 rounded-full bg-gray-200 dark:bg-gray-700" }
                        div { class: "flex-auto truncate",
                            p { class: "text-sm font-medium text-gray-900 dark:text-white", "Duanwu" }
                            p { class: "text-xs text-gray-500 dark:text-gray-400", "duanwu@example.com" }
                        }
                    }
                }

                nav { class: "flex-1 min-h-0 space-y-1 p-2 overflow-y-auto",
                    NavItem { href: "#", label: "JSON Formatter", icon: "{{}}", active: true }
                }
            }

            main { class: "flex-1 min-w-0 h-full flex flex-col overflow-hidden",

                header { class: "flex h-14 flex-shrink-0 items-center gap-4 border-b border-gray-200 bg-white px-6 dark:border-gray-800 dark:bg-gray-950 select-none",
                    h1 { class: "text-sm font-semibold text-gray-900 dark:text-white", "JSON Formatter" }
                }

                div { id: "main-scroll", class: "flex-1 min-h-0 w-full overflow-y-auto p-6 select-text",
                    JsonFormatter{}
                }
            }
        }
    }
}

/// Single item in the sidebar navigation.
#[component]
fn NavItem(href: String, label: String, icon: String, active: bool) -> Element {
    let base = "flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors";
    let state = if active {
        "bg-gray-100 text-gray-900 dark:bg-gray-800 dark:text-white"
    } else {
        "text-gray-600 hover:bg-gray-50 hover:text-gray-900 dark:text-gray-400 \
         dark:hover:bg-gray-800 dark:hover:text-white"
    };
    rsx! {
        a { href: "{href}", class: "{base} {state}",
            span { class: "font-mono text-base", "{icon}" }
            "{label}"
        }
    }
}

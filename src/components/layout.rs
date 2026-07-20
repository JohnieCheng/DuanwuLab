use dioxus::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

use crate::components::base64::Base64;
use crate::components::common::nav_item::NavItem;
use crate::components::json_viewer::JsonFormatter;
use crate::components::password_gen::PasswordGen;
use crate::components::unix_timestamp::UnixTimestamp;
use crate::components::url_codec::UrlCodec;

#[derive(Clone, Copy, PartialEq, EnumIter)]
pub enum Page {
    JsonFormatter,
    Base64,
    UnixTimestamp,
    UrlCodec,
    PasswordGen,
}

impl Page {
    fn label(&self) -> &'static str {
        match self {
            Page::JsonFormatter => "JSON Formatter",
            Page::Base64 => "Base64",
            Page::UnixTimestamp => "Unix Timestamp",
            Page::UrlCodec => "URL Codec",
            Page::PasswordGen => "Password Gen",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Page::JsonFormatter => "{}",
            Page::Base64 => "64",
            Page::UnixTimestamp => "ts",
            Page::UrlCodec => "%%",
            Page::PasswordGen => "pw",
        }
    }

    fn render(&self) -> Element {
        match self {
            Page::JsonFormatter => rsx! { JsonFormatter{} },
            Page::Base64 => rsx! { Base64{} },
            Page::UnixTimestamp => rsx! { UnixTimestamp{} },
            Page::UrlCodec => rsx! { UrlCodec{} },
            Page::PasswordGen => rsx! { PasswordGen{} },
        }
    }
}

/// Top-level layout: sidebar navigation + main content area.
#[component]
pub fn SidebarLayout(page: Signal<Page>) -> Element {
    let mut search = use_signal(String::new);

    let filtered = use_memo(move || -> Vec<Page> {
        let q = search.read();
        let q = q.trim();
        if q.is_empty() {
            return Page::iter().collect();
        }
        // Lowercase search terms once, outside the filter loop.
        let terms: Vec<String> = q.split_whitespace().map(|t| t.to_lowercase()).collect();
        Page::iter()
            .filter(|p| {
                let label = p.label().to_lowercase();
                terms.iter().all(|t| label.contains(t.as_str()))
            })
            .collect()
    });

    let current = *page.read();

    rsx! {
        div { class: "w-screen h-screen flex overflow-hidden bg-gray-50 dark:bg-gray-900 select-none",

            aside { class: "w-56 h-full flex flex-col flex-shrink-0 overflow-hidden border-r border-gray-200 bg-white dark:border-gray-800 dark:bg-gray-950 select-none",
                div { class: "p-3 flex-shrink-0",
                    input {
                        class: "w-full rounded-lg border-0 bg-transparent px-3 py-2 text-sm text-gray-500 placeholder-gray-400 focus:outline-none dark:text-gray-400 dark:placeholder-gray-500",
                        r#type: "text",
                        placeholder: "Filter...",
                        spellcheck: false,
                        value: "{search}",
                        oninput: move |e: FormEvent| search.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                let list = filtered.read();
                                if !list.is_empty() {
                                    page.set(list[0]);
                                }
                            }
                        },
                    }
                }

                nav { class: "flex-1 min-h-0 space-y-1 p-2 overflow-y-auto",
                    if filtered.read().is_empty() {
                        div { class: "px-3 py-8 text-center text-xs text-gray-400 dark:text-gray-500",
                            "No tools found"
                        }
                    } else {
                        for &pg in filtered.read().iter() {
                            NavItem {
                                key: "{pg.label()}",
                                label: pg.label(),
                                icon: pg.icon(),
                                active: current == pg,
                                onclick: move |_| page.set(pg),
                            }
                        }
                    }
                }
            }

            main { class: "flex-1 min-w-0 h-full flex flex-col overflow-hidden",

                header { class: "flex h-14 flex-shrink-0 items-center gap-4 border-b border-gray-200 bg-white px-6 dark:border-gray-800 dark:bg-gray-950 select-none",
                    h1 { class: "text-sm font-semibold text-gray-900 dark:text-white", "{current.label()}" }
                }

                div { id: "main-scroll", class: "flex-1 min-h-0 w-full overflow-y-auto p-6 select-text",
                    { current.render() }
                }
            }
        }
    }
}

use std::collections::{HashMap, HashSet};

use dioxus::document::eval;
use dioxus::prelude::*;
use serde_json::Value;

use crate::components::common::button::{Button, ButtonVariant};
use crate::context::error::GlobalErrorContext;
use crate::utils::safe_invoke::safe_invoke;

mod components;
mod constants;
mod input;
mod node;
mod search;
mod types;

use constants::*;
use input::JsonInputEditor;
use node::JsonNode;
use search::find_json_matches;
use types::{FormatArgs, SearchContext, SearchResultMap};

#[component]
pub fn JsonFormatter() -> Element {
    let input_text = use_signal(String::new);
    let mut output = use_signal(String::new);
    let mut smart_repair = use_signal(|| false);
    let mut auto_format_enabled = use_signal(|| false);
    let mut copied = use_signal(|| false);

    let mut search_input = use_signal(String::new);
    let mut search_query = use_signal(String::new);
    let mut active_index = use_signal(|| 0);

    use_effect(move || {
        let q = search_input.read().clone();
        if q.is_empty() {
            if !search_query.read().is_empty() {
                search_query.set(String::new());
                active_index.set(0);
            }
            return;
        }

        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(SEARCH_DEBOUNCE_MS)).await;
            if search_input.read().as_str() == q.as_str() {
                search_query.set(q);
                active_index.set(0);
            }
        });
    });

    use_effect(move || {
        let text = input_text.read().clone();
        if text.trim().is_empty() {
            smart_repair.set(false);
            return;
        }
        if *smart_repair.read() {
            return;
        }
        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(AUTO_REPAIR_DEBOUNCE_MS))
                .await;
            if input_text.read().as_str() == text.as_str()
                && serde_json::from_str::<Value>(&text).is_err()
            {
                smart_repair.set(true);
            }
        });
    });

    let parsed_json = use_memo(move || {
        let raw = output.read();
        let clean = raw.split("\n\n// Repaired").next().unwrap_or(&raw);
        serde_json::from_str::<Value>(clean).ok()
    });

    let search_results = use_memo(move || {
        let mut list = Vec::new();
        let mut path_to_match = HashMap::new();
        let mut active_ancestor_paths = HashSet::new();
        let q = search_query.read();
        let active_idx = *active_index.read();

        if !q.is_empty()
            && let Some(v) = parsed_json.read().as_ref()
        {
            let q_low = q.to_lowercase();
            find_json_matches(v, &q_low, "", &mut list);

            for (idx, m) in list.iter().enumerate() {
                path_to_match.insert(m.path.clone(), (idx, m.is_key));
            }

            if let Some(active_match) = list.get(active_idx) {
                let mut temp_path = String::new();
                for part in active_match.path.split('/').filter(|s| !s.is_empty()) {
                    temp_path.push('/');
                    temp_path.push_str(part);
                    active_ancestor_paths.insert(temp_path.clone());
                }
            }
        }

        SearchResultMap { list, path_to_match, active_ancestor_paths }
    });

    use_context_provider(move || SearchContext {
        query: search_query,
        search_results,
        active_index,
    });

    let mut format_text = move |text: String| {
        if text.trim().is_empty() {
            return;
        }
        output.set(String::new());
        search_input.set(String::new());
        search_query.set(String::new());
        active_index.set(0);

        let use_repair = *smart_repair.read();
        let error_ctx = use_context::<GlobalErrorContext>();
        spawn(async move {
            let args = &FormatArgs { input: text.clone(), repair: use_repair };
            if let Some(res_string) = safe_invoke::<String, _>("format_json", args, error_ctx).await
            {
                output.set(res_string);
            }
        });
    };

    let total_matches = search_results.read().list.len();
    let current_match_display = if total_matches == 0 { 0 } else { *active_index.read() + 1 };

    use_effect(move || {
        let active_idx = *active_index.read();
        let results = search_results.read();

        if !results.list.is_empty() {
            let js_code = format!(
                r#"
                (function() {{
                    const targetId = "search-match-{}";
                    let attempts = 0;
                    function doScroll() {{
                        const el = document.getElementById(targetId);
                        if (el) {{
                            el.scrollIntoView({{ behavior: "smooth", block: "center" }});
                            return;
                        }}
                        attempts++;
                        if (attempts < 30) {{ setTimeout(doScroll, 50); }}
                    }}
                    setTimeout(doScroll, 20);
                }})();
                "#,
                active_idx
            );
            eval(&js_code);
        }
    });

    // Global keyboard shortcut: Ctrl/Cmd+Enter triggers Format when focus is in the input textarea.
    use_effect(move || {
        let js_code = r#"
            (function() {
                if (window._jsonFormatterKeyHandler) { window.removeEventListener('keydown', window._jsonFormatterKeyHandler); }
                window._jsonFormatterKeyHandler = function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.code === 'Enter') {
                        const active = document.activeElement;
                        if (active && active.id === 'json-input') {
                            e.preventDefault();
                            document.getElementById('format-btn').click();
                        }
                    }
                };
                window.addEventListener('keydown', window._jsonFormatterKeyHandler);
            })();
        "#;
        eval(js_code);
    });

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400", "Input" }
                JsonInputEditor { input_text, on_auto_format: move |val| {
                    if *auto_format_enabled.read() {
                        format_text(val);
                    }
                }, on_clear: move |_| {
                    output.set(String::new());
                    smart_repair.set(false);
                } }
            }

            div { class: "flex items-center gap-4",
                label { class: "flex cursor-pointer select-none items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    input { r#type: "checkbox", checked: *smart_repair.read(), onchange: move |e| smart_repair.set(e.value() == "true"), class: "rounded border-gray-300" }
                    "Smart repair"
                }
                label { class: "flex cursor-pointer select-none items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    input { r#type: "checkbox", checked: *auto_format_enabled.read(), onchange: move |e| auto_format_enabled.set(e.value() == "true"), class: "rounded border-gray-300" }
                    "Auto-format"
                }
                div { class: "flex-1" }
                Button { label: "Format", variant: ButtonVariant::Solid, id: "format-btn", onclick: move |_| {
                    let text = input_text.read().clone();
                    if !text.trim().is_empty() { format_text(text); }
                }}
            }

            if !output.read().is_empty() {
                div { class: "flex flex-col relative",
                    div { class: "flex h-8 items-center", label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 px-1", "Output" } }
                    div { class: "sticky top-0 z-10 flex w-full justify-end -mt-8 h-8 bg-transparent items-center pointer-events-none pr-3",
                        div { class: "flex items-center gap-3 pointer-events-auto py-1 dark:bg-gray-900 rounded-md pl-2",
                            div { class: "flex items-center gap-1.5 rounded-md border border-gray-200 bg-white px-2 py-0.5 text-xs dark:border-gray-700 dark:bg-gray-950 transition-colors",
                                span { class: "text-gray-400 select-none text-[11px]", "🔍" }
                                input {
                                    r#type: "text", placeholder: "Search key/value...", value: "{search_input}",
                                    oninput: move |e| search_input.set(e.value()),
                                    class: "w-36 md:w-44 bg-transparent py-0.5 outline-none text-gray-900 dark:text-gray-100 font-mono text-[11px] select-text",
                                }
                                span { class: "text-[10px] font-mono text-gray-400 border-r border-gray-200 dark:border-gray-700 pr-2 h-3 flex items-center", "{current_match_display}/{total_matches}" }
                                button { class: "text-gray-400 hover:text-gray-600 font-bold px-0.5 text-[11px]", onclick: move |_| { if total_matches > 0 { active_index.with_mut(|idx| if *idx == 0 { *idx = total_matches - 1; } else { *idx -= 1; }); } }, "↑" }
                                button { class: "text-gray-400 hover:text-gray-600 font-bold px-0.5 text-[11px]", onclick: move |_| { if total_matches > 0 { active_index.with_mut(|idx| *idx = (*idx + 1) % total_matches); } }, "↓" }
                            }
                            button {
                                class: "rounded border border-gray-200 dark:border-gray-700 px-2 py-0.5 text-[11px] text-gray-400 hover:text-gray-600 hover:border-gray-300 dark:text-gray-500 dark:hover:text-gray-300 bg-white dark:bg-gray-950 transition-colors select-none",
                                onclick: move |_| { eval("document.getElementById('main-scroll').scrollTo({top:0,behavior:'smooth'})"); },
                                "↑ Top"
                            }
                            button {
                                class: "rounded border border-gray-200 dark:border-gray-700 px-2 py-0.5 text-[11px] transition-all duration-200 select-none",
                                class: if *copied.read() {
                                    "text-emerald-600 border-emerald-200 dark:text-emerald-400 dark:border-emerald-800"
                                } else {
                                    "text-gray-400 hover:text-gray-600 hover:border-gray-300 dark:text-gray-500 dark:hover:text-gray-300 bg-white dark:bg-gray-950"
                                },
                                onclick: move |_| {
                                    let raw = output.read();
                                    let clean = raw.split("\n\n// Repaired").next().unwrap_or(&raw);
                                    let escaped = serde_json::to_string(clean).unwrap();
                                    eval(&format!("navigator.clipboard.writeText({})", escaped));
                                    copied.set(true);
                                    spawn(async move {
                                        gloo_timers::future::sleep(std::time::Duration::from_millis(COPY_FEEDBACK_MS)).await;
                                        copied.set(false);
                                    });
                                },
                                if *copied.read() { "✓ Copied" } else { "Copy" }
                            }
                        }
                    }
                    div { class: "mt-2",
                        div {
                            id: "json-tree-container",
                            class: "overflow-auto rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm leading-relaxed text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 select-text",
                            {
                                match parsed_json.read().clone() {
                                    Some(v) => rsx! { JsonNode { value: v, depth: 0, is_last: true, key_name: None, index: None, path: "".to_string() } },
                                    None => rsx! { pre { class: "whitespace-pre-wrap select-text", "{output}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

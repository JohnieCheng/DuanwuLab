use crate::context::error::GlobalErrorContext;
use crate::utils::safe_invoke::safe_invoke;
use dioxus::document::eval;
use dioxus::prelude::*;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct FormatArgs {
    input: String,
    repair: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct JsonMatch {
    path: String,
    is_key: bool,
}

#[derive(Clone, Copy)]
struct SearchContext {
    query: Signal<String>,
    matches: Memo<Vec<JsonMatch>>,
    active_index: Signal<usize>,
}

#[component]
pub fn JsonFormatter() -> Element {
    let mut input = use_signal(String::new);
    let mut output = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut repair = use_signal(|| false);
    let mut show_tree = use_signal(|| true);

    let mut search_query = use_signal(String::new);
    let mut active_index = use_signal(|| 0);

    let parsed_json = use_memo(move || {
        let raw = output.read();
        let clean = raw.split("\n\n// Repaired").next().unwrap_or(&raw);
        serde_json::from_str::<Value>(clean).ok()
    });

    let matches = use_memo(move || {
        let mut list = Vec::new();
        let q = search_query.read();
        if !q.is_empty() {
            if let Some(v) = parsed_json.read().as_ref() {
                let q_low = q.to_lowercase();
                find_json_matches(v, &q_low, "", &mut list);
            }
        }
        list
    });

    use_context_provider(move || SearchContext { query: search_query, matches, active_index });

    let format = move |_| {
        let input_val = input.read().clone();
        let use_repair = *repair.read();
        error.set(String::new());
        output.set(String::new());
        search_query.set(String::new());
        active_index.set(0);

        let error_ctx = use_context::<GlobalErrorContext>();
        spawn(async move {
            let args = &FormatArgs { input: input_val, repair: use_repair };
            if let Some(res_string) = safe_invoke::<String, _>("format_json", args, error_ctx).await
            {
                output.set(res_string);
            }
        });
    };

    let total_matches = matches.read().len();
    let current_match_display = if total_matches == 0 { 0 } else { *active_index.read() + 1 };

    use_effect(move || {
        let active_idx = *active_index.read();
        let total = matches.read().len();

        if total > 0 {
            let js_code = format!(
                r#"
                (function() {{
                    const targetId = "search-match-{}";
                    let attempts = 0;
                    let lastTop = -1;
                    let stableCount = 0;

                    function doScroll() {{
                        const el = document.getElementById(targetId);
                        if (el) {{
                            const rect = el.getBoundingClientRect();
                            const currentTop = rect.top;

                            if (lastTop === currentTop && currentTop !== 0) {{
                                stableCount++;
                            }} else {{
                                stableCount = 0;
                            }}
                            lastTop = currentTop;

                            if (stableCount >= 4) {{
                                el.scrollIntoView({{ behavior: "smooth", block: "center" }});
                                return;
                            }}
                        }}

                        attempts++;
                        if (attempts < 60) {{
                            setTimeout(doScroll, 40);
                        }}
                    }}
                    setTimeout(doScroll, 40);
                }})();
                "#,
                active_idx
            );
            eval(&js_code);
        }
    });

    // ✨ 优化后的全局键盘事件拦截
    use_effect(move || {
        let js_code = r#"
            (function() {
                if (window._jsonFormatterKeyHandler) {
                    window.removeEventListener('keydown', window._jsonFormatterKeyHandler);
                }
                window._jsonFormatterKeyHandler = function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.code === 'KeyA') {
                        const active = document.activeElement;
                        if (active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA')) {
                            return;
                        }

                        const container = document.getElementById('json-tree-container') ||
                                          document.getElementById('json-raw-container') ||
                                          document.getElementById('json-raw-fallback-container');
                        if (container) {
                            e.preventDefault();
                            const range = document.createRange();
                            range.selectNodeContents(container);
                            const sel = window.getSelection();
                            sel.removeAllRanges();
                            sel.addRange(range);
                        } else {
                            const inputTextArea = document.querySelector('textarea');
                            if (inputTextArea) {
                                e.preventDefault();
                                inputTextArea.focus();
                                inputTextArea.select();
                            } else {
                                e.preventDefault();
                            }
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
                textarea {
                    class: "min-h-[200px] w-full resize-y rounded-lg border border-gray-200 bg-white p-4 font-mono text-sm text-gray-900 placeholder:text-gray-400 focus:border-gray-400 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 dark:placeholder:text-gray-500 select-text",
                    placeholder: "Paste JSON here...",
                    value: "{input}",
                    oninput: move |e| input.set(e.value()),
                }
            }
            div { class: "flex items-center gap-4",
                label { class: "flex cursor-pointer select-none items-center gap-2 text-sm text-gray-600 dark:text-gray-400",
                    input {
                        r#type: "checkbox", checked: *repair.read(),
                        onchange: move |e| repair.set(e.value() == "true"),
                        class: "rounded border-gray-300 dark:border-gray-600",
                    }
                    "Smart repair"
                }
                div { class: "flex-1" }
                button {
                    class: "rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-gray-800 dark:bg-white dark:text-gray-900 dark:hover:bg-gray-200",
                    onclick: format, "Format"
                }
            }
            if !error.read().is_empty() {
                pre { class: "whitespace-pre-wrap rounded-lg bg-red-50 p-4 font-mono text-sm text-red-600 dark:bg-red-950 dark:text-red-400 select-text",
                    "{error}"
                }
            }
            if !output.read().is_empty() {
                div { class: "flex flex-col relative",
                    div { class: "flex h-8 items-center",
                        label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400 px-1", "Output" }
                    }
                    div { class: "sticky top-0 z-10 flex w-full justify-end -mt-8 h-8 bg-transparent items-center pointer-events-none pr-3",
                        div { class: "flex items-center gap-3 pointer-events-auto py-1 dark:bg-gray-900 rounded-md pl-2",
                            if *show_tree.read() {
                                div { class: "flex items-center gap-1.5 rounded-md border border-gray-200 bg-white px-2 py-0.5 text-xs focus-within:border-gray-400 dark:border-gray-700 dark:bg-gray-950 focus-within:dark:border-gray-500 transition-colors",
                                    span { class: "text-gray-400 dark:text-gray-500 select-none text-[11px]", "🔍" }
                                    input {
                                        r#type: "text",
                                        placeholder: "Search key/value...",
                                        value: "{search_query}",
                                        oninput: move |e| {
                                            search_query.set(e.value());
                                            active_index.set(0);
                                        },
                                        class: "w-36 md:w-44 bg-transparent py-0.5 outline-none text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-600 font-mono text-[11px] select-text",
                                    }
                                    span { class: "text-[10px] font-mono text-gray-400 dark:text-gray-500 select-none border-r border-gray-200 dark:border-gray-700 pr-2 h-3 flex items-center",
                                        "{current_match_display}/{total_matches}"
                                    }
                                    button {
                                        class: "text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 font-bold px-0.5 transition-colors text-[11px]",
                                        onclick: move |_| {
                                            if total_matches > 0 {
                                                active_index.with_mut(|idx| {
                                                    if *idx == 0 { *idx = total_matches - 1; } else { *idx -= 1; }
                                                });
                                            }
                                        },
                                        "↑"
                                    }
                                    button {
                                        class: "text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 font-bold px-0.5 transition-colors text-[11px]",
                                        onclick: move |_| {
                                            if total_matches > 0 {
                                                active_index.with_mut(|idx| *idx = (*idx + 1) % total_matches);
                                            }
                                        },
                                        "↓"
                                    }
                                }
                            }
                            button {
                                class: "rounded border border-gray-200 dark:border-gray-700 px-2 py-0.5 text-xs text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors bg-white dark:bg-gray-950",
                                onclick: move |_| show_tree.with_mut(|v| *v = !*v),
                                if *show_tree.read() { "Raw" } else { "Tree" }
                            }
                        }
                    }
                    div { class: "mt-2",
                        if *show_tree.read() {
                            div {
                                id: "json-tree-container",
                                class: "overflow-auto rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm leading-relaxed text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 select-text",
                                {
                                    match parsed_json.read().clone() {
                                        Some(v) => rsx! { JsonNode { value: v, depth: 0, is_last: true, key_name: None, index: None, path: "".to_string() } },
                                        None => rsx! { pre { id: "json-raw-fallback-container", class: "whitespace-pre-wrap select-text", "{output}" } }
                                    }
                                }
                            }
                        } else {
                            pre {
                                id: "json-raw-container",
                                class: "overflow-auto whitespace-pre-wrap rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 select-text",
                                "{output}"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn find_json_matches(value: &Value, q: &str, current_path: &str, matches: &mut Vec<JsonMatch>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let next_path = format!("{}/{}", current_path, k);
                if k.to_lowercase().contains(q) {
                    matches.push(JsonMatch { path: next_path.clone(), is_key: true });
                }
                find_json_matches(v, q, &next_path, matches);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next_path = format!("{}/{}", current_path, i);
                find_json_matches(v, q, &next_path, matches);
            }
        }
        Value::String(s) => {
            if s.to_lowercase().contains(q) {
                matches.push(JsonMatch { path: current_path.to_string(), is_key: false });
            }
        }
        Value::Number(n) => {
            if n.to_string().contains(q) {
                matches.push(JsonMatch { path: current_path.to_string(), is_key: false });
            }
        }
        Value::Bool(b) => {
            if b.to_string().contains(q) {
                matches.push(JsonMatch { path: current_path.to_string(), is_key: false });
            }
        }
        Value::Null => {
            if "null".contains(q) {
                matches.push(JsonMatch { path: current_path.to_string(), is_key: false });
            }
        }
    }
}

fn render_highlighted(
    text: &str,
    match_idx: Option<usize>,
    active_global_idx: usize,
    query: &str,
    default_class: &str,
) -> Element {
    match match_idx {
        Some(idx) => {
            let is_active = idx == active_global_idx;
            let highlight_class = if is_active {
                "bg-orange-500 text-white font-bold rounded px-0.5 ring-2 ring-orange-300 shadow-sm transition-all duration-200"
            } else {
                "bg-yellow-200 dark:bg-yellow-800 text-gray-900 dark:text-gray-100 rounded px-0.5"
            };

            if !query.is_empty() {
                if let Some(start_idx) = text.to_lowercase().find(&query.to_lowercase()) {
                    let end_idx = start_idx + query.len();
                    let prefix = &text[..start_idx];
                    let matched = &text[start_idx..end_idx];
                    let suffix = &text[end_idx..];

                    return rsx! {
                        span { class: "{default_class}",
                            "{prefix}"
                            span {
                                id: format!("search-match-{}", idx),
                                class: "{highlight_class}",
                                "{matched}"
                            }
                            "{suffix}"
                        }
                    };
                }
            }

            rsx! {
                span {
                    id: format!("search-match-{}", idx),
                    class: "{highlight_class}",
                    "{text}"
                }
            }
        }
        None => rsx! { span { class: "{default_class}", "{text}" } },
    }
}

fn get_object_preview(map: &serde_json::Map<String, Value>) -> Option<String> {
    let fuzzy_match = map.iter().find(|(k, v)| {
        let k_lower = k.to_lowercase();
        let is_primitive = v.is_string() || v.is_number() || v.is_boolean();
        is_primitive
            && (k_lower.contains("name") || k_lower.contains("id") || k_lower.contains("code"))
    });
    let final_match = fuzzy_match
        .or_else(|| map.iter().find(|(_, v)| v.is_string() || v.is_number() || v.is_boolean()));
    if let Some((key, val)) = final_match {
        let val_str = match val {
            Value::String(s) => format!("\"{}\"", s),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => return None,
        };
        return if map.len() > 1 {
            Some(format!("\"{}\": {}, ...", key, val_str))
        } else {
            Some(format!("\"{}\": {}", key, val_str))
        };
    }
    None
}

#[component]
fn JsonNode(
    value: Value,
    depth: u8,
    is_last: bool,
    key_name: Option<String>,
    index: Option<usize>,
    path: String,
) -> Element {
    let mut collapsed = use_signal(|| depth > 0);
    let indent_px = (depth as usize) * 16;
    let search = use_context::<SearchContext>();

    let matches_list = search.matches.read();
    let query_str = search.query.read();
    let active_global_idx = *search.active_index.read();

    let effect_path = path.clone();
    use_effect(move || {
        let _q = search.query.read();
        let list = search.matches.read();
        if !list.is_empty() {
            let has_matching_child = list.iter().any(|m| {
                if effect_path.is_empty() {
                    true
                } else {
                    m.path == effect_path || m.path.starts_with(&format!("{}/", effect_path))
                }
            });

            if has_matching_child {
                collapsed.set(false);
            }
        }
    });

    let focus_path = path.clone();
    use_effect(move || {
        let active_idx = *search.active_index.read();
        let list = search.matches.read();

        if !list.is_empty() {
            let is_active_target = list.get(active_idx).map_or(false, |m| {
                if focus_path.is_empty() {
                    true
                } else {
                    m.path == focus_path || m.path.starts_with(&format!("{}/", focus_path))
                }
            });

            if is_active_target {
                collapsed.set(false);
            }
        }
    });

    let key_match_idx = if key_name.is_some() {
        matches_list.iter().position(|m| m.is_key && &m.path == &path)
    } else {
        None
    };
    let val_match_idx = matches_list.iter().position(|m| !m.is_key && &m.path == &path);

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "{{}}" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else if *collapsed.read() {
                let preview_text = get_object_preview(&map);
                let suffix = if map.len() != 1 { "s" } else { "" };
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(false),
                            "+"
                        }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| collapsed.set(false),
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "{{" }
                            if let Some(preview) = preview_text {
                                span { class: "text-gray-500 dark:text-gray-400 italic text-xs mx-1 bg-gray-100 dark:bg-gray-800 px-1 rounded", "{preview}" }
                            } else {
                                span { class: "text-xs italic text-gray-400 mx-1", "{map.len()} key{suffix}" }
                            }
                            span { class: "text-gray-400", "}}" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else {
                let len = map.len();
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(true),
                            "-"
                        }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "{{" }
                        }
                    }
                    {
                        map.iter().enumerate().map(|(i, (k, v))| {
                            let child_path = format!("{}/{}", path, k);
                            rsx! {
                                JsonNode {
                                    key: "{child_path}",
                                    value: v.clone(),
                                    depth: depth + 1,
                                    is_last: i == len - 1,
                                    key_name: Some(k.clone()),
                                    index: None,
                                    path: child_path
                                }
                            }
                        })
                    }
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            span { class: "text-gray-400", "}}" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            }
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "[]" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else if *collapsed.read() {
                let suffix = if arr.len() != 1 { "s" } else { "" };
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(false),
                            "+"
                        }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| collapsed.set(false),
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "[ " }
                            span { class: "text-xs italic text-gray-400", "{arr.len()} item{suffix}" }
                            span { class: "text-gray-400", " ]" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else {
                let len = arr.len();
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(true),
                            "-"
                        }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name {
                                span { class: "text-blue-600 dark:text-blue-400", "\"" }
                                { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                                span { class: "text-blue-600 dark:text-blue-400", "\": " }
                            }
                            span { class: "text-gray-400", "[" }
                        }
                    }
                    {
                        arr.iter().enumerate().map(|(i, item)| {
                            let child_path = format!("{}/{}", path, i);
                            rsx! {
                                JsonNode {
                                    key: "{child_path}",
                                    value: item.clone(),
                                    depth: depth + 1,
                                    is_last: i == len - 1,
                                    key_name: None,
                                    index: Some(i),
                                    path: child_path
                                }
                            }
                        })
                    }
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            span { class: "text-gray-400", "]" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            }
        }
        _ => {
            rsx! {
                div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                    div { class: "w-4 flex-shrink-0 select-none" }
                    div { class: "select-text flex-1",
                        if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                        if let Some(k) = &key_name {
                            span { class: "text-blue-600 dark:text-blue-400", "\"" }
                            { render_highlighted(k, key_match_idx, active_global_idx, &query_str, "text-blue-600 dark:text-blue-400") }
                            span { class: "text-blue-600 dark:text-blue-400", "\": " }
                        }
                        match value {
                            Value::String(s) => rsx! {
                                span { class: "text-green-600 dark:text-green-400", "\"" }
                                { render_highlighted(&s, val_match_idx, active_global_idx, &query_str, "text-green-600 dark:text-green-400") }
                                span { class: "text-green-600 dark:text-green-400", "\"" }
                            },
                            Value::Number(n) => rsx! {
                                { render_highlighted(&n.to_string(), val_match_idx, active_global_idx, &query_str, "text-amber-600 dark:text-amber-400") }
                            },
                            Value::Bool(b) => rsx! {
                                { render_highlighted(&b.to_string(), val_match_idx, active_global_idx, &query_str, "text-purple-600 dark:text-purple-400") }
                            },
                            Value::Null => rsx! {
                                { render_highlighted("null", val_match_idx, active_global_idx, &query_str, "text-gray-400 italic") }
                            },
                            _ => unreachable!()
                        }
                        if !is_last { span { class: "text-gray-400", "," } }
                    }
                }
            }
        }
    }
}

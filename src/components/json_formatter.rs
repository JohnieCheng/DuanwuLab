use dioxus::prelude::*;
use serde::Serialize;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize)]
struct FormatArgs {
    input: String,
    repair: bool,
}

#[component]
pub fn JsonFormatter() -> Element {
    let mut input = use_signal(String::new);
    let mut output = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut repair = use_signal(|| false);
    let mut show_tree = use_signal(|| true);

    let format = move |_| {
        let input_val = input.read().clone();
        let use_repair = *repair.read();
        error.set(String::new());
        output.set(String::new());

        spawn(async move {
            let args =
                serde_wasm_bindgen::to_value(&FormatArgs { input: input_val, repair: use_repair })
                    .unwrap();
            let raw = invoke("format_json", args).await;
            if let Some(s) = raw.as_string() {
                output.set(s);
            } else if let Some(err) = raw.as_f64() {
                error.set(format!("Error code: {}", err));
            } else {
                error.set("Unexpected response from backend".into());
            }
        });
    };

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6",
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400", "Input" }
                textarea {
                    class: "min-h-[200px] w-full resize-y rounded-lg border border-gray-200 bg-white p-4 font-mono text-sm text-gray-900 placeholder:text-gray-400 focus:border-gray-400 focus:outline-none dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 dark:placeholder:text-gray-500",
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
                pre { class: "whitespace-pre-wrap rounded-lg bg-red-50 p-4 font-mono text-sm text-red-600 dark:bg-red-950 dark:text-red-400",
                    "{error}"
                }
            }
            if !output.read().is_empty() {
                div { class: "flex flex-col gap-2",
                    div { class: "flex items-center gap-2",
                        label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400", "Output" }
                        button {
                            class: "ml-auto rounded px-2 py-0.5 text-xs text-gray-500 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-800 transition-colors",
                            onclick: move |_| show_tree.with_mut(|v| *v = !*v),
                            if *show_tree.read() { "Raw" } else { "Tree" }
                        }
                    }
                    if *show_tree.read() {
                        div { class: "overflow-auto rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm leading-relaxed text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100",
                            { render_tree(output.read().clone()) }
                        }
                    } else {
                        pre { class: "overflow-auto whitespace-pre-wrap rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100",
                            "{output}"
                        }
                    }
                }
            }
        }
    }
}

fn render_tree(raw: String) -> Element {
    let clean = raw.split("\n\n// Repaired").next().unwrap_or(&raw);
    match serde_json::from_str::<Value>(clean) {
        Ok(v) => {
            rsx! { JsonNode { value: v, depth: 0, is_last: true, key_name: None, index: None } }
        }
        Err(_) => rsx! { pre { class: "whitespace-pre-wrap", "{raw}" } },
    }
}

// Smartly extract the object's "identifying fields" as the collapsed preview
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
            Value::String(s) => {
                format!("\"{}\"", s)
            }
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
) -> Element {
    let mut collapsed = use_signal(|| depth > 0);
    let indent_px = (depth as usize) * 16;

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
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
                        // 修正：加入 h-5 焊死高度，让 items-center 可以在行高内垂直居中
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(false),
                            "+"
                        }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| collapsed.set(false),
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
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
                        // 修正：加入 h-5
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(true),
                            "-"
                        }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
                            span { class: "text-gray-400", "{{" }
                        }
                    }
                    {
                        map.iter().enumerate().map(|(i, (k, v))| {
                            rsx! {
                                JsonNode {
                                    key: "{k}",
                                    value: v.clone(),
                                    depth: depth + 1,
                                    is_last: i == len - 1,
                                    key_name: Some(k.clone()),
                                    index: None
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
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
                            span { class: "text-gray-400", "[]" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else if *collapsed.read() {
                let suffix = if arr.len() != 1 { "s" } else { "" };
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        // 修正：加入 h-5
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(false),
                            "+"
                        }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| collapsed.set(false),
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
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
                        // 修正：加入 h-5
                        div {
                            class: "w-4 h-5 flex-shrink-0 select-none cursor-pointer text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 inline-flex items-center justify-center font-bold text-xs",
                            onclick: move |_| collapsed.set(true),
                            "-"
                        }
                        div { class: "select-text flex-1",
                            if let Some(idx) = index { span { class: "text-gray-400 dark:text-gray-500 mr-1.5 font-bold", "{idx}:" } }
                            if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
                            span { class: "text-gray-400", "[" }
                        }
                    }
                    {
                        arr.iter().enumerate().map(|(i, item)| {
                            rsx! {
                                JsonNode {
                                    key: "{i}",
                                    value: item.clone(),
                                    depth: depth + 1,
                                    is_last: i == len - 1,
                                    key_name: None,
                                    index: Some(i)
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
                        if let Some(k) = &key_name { span { class: "text-blue-600 dark:text-blue-400", "\"{k}\"" } span { class: "text-gray-400", ": " } }
                        match value {
                            Value::String(s) => rsx! { span { class: "text-green-600 dark:text-green-400", "\"{s}\"" } },
                            Value::Number(n) => rsx! { span { class: "text-amber-600 dark:text-amber-400", "{n}" } },
                            Value::Bool(b) => rsx! { span { class: "text-purple-600 dark:text-purple-400", "{b}" } },
                            Value::Null => rsx! { span { class: "text-gray-400 italic", "null" } },
                            _ => unreachable!()
                        }
                        if !is_last { span { class: "text-gray-400", "," } }
                    }
                }
            }
        }
    }
}

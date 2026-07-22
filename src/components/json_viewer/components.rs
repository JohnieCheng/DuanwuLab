use dioxus::prelude::*;
use serde_json::Value;

use super::types::HighlightProps;

#[component]
pub(crate) fn RenderHighlighted(props: HighlightProps) -> Element {
    match props.match_idx {
        Some(idx) => {
            let is_active = idx == props.active_global_idx;
            if !props.query.is_empty()
                && let Some(start_idx) = props.text.to_lowercase().find(&props.query.to_lowercase())
            {
                let end_idx = start_idx + props.query.len();
                return rsx! {
                    span {
                        class: "{props.default_class}",
                        "{&props.text[..start_idx]}"
                        span {
                            id: format!("search-match-{}", idx),
                            class: "rounded px-0.5 shadow-sm",
                            class: if is_active { "bg-orange-500 text-white font-bold ring-2 ring-orange-300" },
                            class: if !is_active { "bg-yellow-200 dark:bg-yellow-800 text-gray-900 dark:text-gray-100" },
                            "{&props.text[start_idx..end_idx]}"
                        }
                        "{&props.text[end_idx..]}"
                    }
                };
            }
            rsx! {
                span {
                    id: format!("search-match-{}", idx),
                    class: "rounded px-0.5 shadow-sm",
                    class: if is_active { "bg-orange-500 text-white font-bold ring-2 ring-orange-300" },
                    class: if !is_active { "bg-yellow-200 dark:bg-yellow-800 text-gray-900 dark:text-gray-100" },
                    "{props.text}"
                }
            }
        }
        None => rsx! { span { class: "{props.default_class}", "{props.text}" } },
    }
}

#[component]
pub(crate) fn NodeHeader(
    index: Option<usize>,
    key_name: Option<String>,
    key_match_idx: Option<usize>,
    active_global_idx: usize,
    query_str: String,
) -> Element {
    rsx! {
        if let Some(idx) = index {
            span { class: "text-gray-400 mr-1.5 font-bold", "{idx}:" }
        }
        if let Some(k) = key_name {
            span { class: "text-blue-600 dark:text-blue-400", "\"" }
            RenderHighlighted { text: k.clone(), match_idx: key_match_idx, active_global_idx, query: query_str.clone(), default_class: "text-blue-600 dark:text-blue-400".to_string() }
            span { class: "text-blue-600 dark:text-blue-400", "\": " }
        }
    }
}

#[component]
pub(crate) fn CollapsedPreview(value: Value) -> Element {
    match value {
        Value::Object(map) => {
            let preview_text = get_object_preview(&map);
            rsx! {
                span { class: "text-gray-400", "{{" }
                if let Some(preview) = preview_text {
                    span { class: "text-gray-500 italic text-xs mx-1 bg-gray-100 dark:bg-gray-800 px-1 rounded", "{preview}" }
                } else {
                    span { class: "text-xs italic text-gray-400 mx-1", "{map.len()} keys" }
                }
                span { class: "text-gray-400", "}}" }
            }
        }
        Value::Array(arr) => rsx! {
            span { class: "text-gray-400", "[ " }
            span { class: "text-xs italic text-gray-400", "{arr.len()} items" }
            span { class: "text-gray-400", " ]" }
        },
        _ => rsx! {},
    }
}

#[component]
pub(crate) fn PrimitiveValue(
    value: Value,
    val_match_idx: Option<usize>,
    active_global_idx: usize,
    query_str: String,
) -> Element {
    match value {
        Value::String(s) => rsx! {
            span { class: "text-green-600 dark:text-green-400", "\"" }
            RenderHighlighted { text: s, match_idx: val_match_idx, active_global_idx, query: query_str.clone(), default_class: "text-green-600 dark:text-green-400".to_string() }
            span { class: "text-green-600 dark:text-green-400", "\"" }
        },
        Value::Number(n) => rsx! {
            RenderHighlighted { text: n.to_string(), match_idx: val_match_idx, active_global_idx, query: query_str, default_class: "text-amber-600 dark:text-amber-400".to_string() }
        },
        Value::Bool(b) => rsx! {
            RenderHighlighted { text: b.to_string(), match_idx: val_match_idx, active_global_idx, query: query_str, default_class: "text-purple-600 dark:text-purple-400".to_string() }
        },
        Value::Null => rsx! {
            RenderHighlighted { text: "null".to_string(), match_idx: val_match_idx, active_global_idx, query: query_str, default_class: "text-gray-400 italic".to_string() }
        },
        _ => unreachable!(),
    }
}

pub(crate) fn get_object_preview(map: &serde_json::Map<String, Value>) -> Option<String> {
    let fuzzy_match = map.iter().find(|(k, v)| {
        let k_lower = k.to_lowercase();
        (v.is_string() || v.is_number() || v.is_boolean())
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
        return Some(if map.len() > 1 {
            format!("\"{}\": {}, ...", key, val_str)
        } else {
            format!("\"{}\": {}", key, val_str)
        });
    }
    None
}

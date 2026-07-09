use dioxus::prelude::*;
use serde_json::Value;

use super::components::*;
use super::constants::AUTO_EXPAND_LIMIT;
use super::types::SearchContext;

#[component]
pub(super) fn JsonNode(
    value: Value,
    depth: u8,
    is_last: bool,
    key_name: Option<String>,
    index: Option<usize>,
    path: String,
    #[props(default)] auto_expand: bool,
) -> Element {
    let mut manual_collapsed = use_signal(|| None::<bool>);
    let mut is_permanently_expanded = use_signal(|| false);

    let indent_px = (depth as usize) * 16;
    let search = use_context::<SearchContext>();

    let search_res = search.search_results.read();
    let query_str = search.query.read().clone();
    let active_global_idx = *search.active_index.read();

    let is_current_match_path =
        !query_str.is_empty() && search_res.active_ancestor_paths.contains(&path);

    if is_current_match_path && !*is_permanently_expanded.read() {
        *is_permanently_expanded.write() = true;
    }

    let should_expand = depth == 0
        || is_current_match_path
        || *is_permanently_expanded.read()
        || (auto_expand && is_small_container(&value));

    let is_collapsed = match *manual_collapsed.read() {
        Some(manual_val) => manual_val,
        None => !should_expand,
    };

    // Auto-expand small children only when the user explicitly clicked to expand
    let cascade_auto_expand = *manual_collapsed.read() == Some(false);

    let (key_match_idx, val_match_idx) = match search_res.path_to_match.get(&path) {
        Some(&(idx, is_key)) => {
            if is_key {
                (Some(idx), None)
            } else {
                (None, Some(idx))
            }
        }
        None => (None, None),
    };

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1",
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str }
                            span { class: "text-gray-400", "{{}}" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else if is_collapsed {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 h-5 flex-shrink-0 cursor-pointer text-gray-400 hover:text-gray-600 font-bold text-xs inline-flex items-center justify-center", onclick: move |_| manual_collapsed.set(Some(false)), "+" }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| manual_collapsed.set(Some(false)),
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str }
                            CollapsedPreview { value: Value::Object(map) }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else {
                let len = map.len();
                let current_path = path.clone();
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 h-5 flex-shrink-0 cursor-pointer text-gray-400 hover:text-gray-600 font-bold text-xs inline-flex items-center justify-center", onclick: move |_| manual_collapsed.set(Some(true)), "-" }
                        div { class: "select-text flex-1",
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str: query_str.clone() }
                            span { class: "text-gray-400", "{{" }
                        }
                    }
                    { map.iter().enumerate().map(move |(i, (k, v))| {
                        let child_path = if current_path.is_empty() { format!("/{}", k) } else { format!("{}/{}", current_path, k) };
                        let child_depth = depth + 1;
                        let child_is_last = i == len - 1;
                        let auto_expand = cascade_auto_expand && is_small_container(v);
                        rsx! { JsonNode { key: "{child_path}", value: v.clone(), depth: child_depth, is_last: child_is_last, key_name: Some(k.clone()), index: None, path: child_path, auto_expand: auto_expand } }
                    }) }
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1", span { class: "text-gray-400", "}}" } if !is_last { span { class: "text-gray-400", "," } } }
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
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str }
                            span { class: "text-gray-400", "[]" }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else if is_collapsed {
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 h-5 flex-shrink-0 cursor-pointer text-gray-400 hover:text-gray-600 font-bold text-xs inline-flex items-center justify-center", onclick: move |_| manual_collapsed.set(Some(false)), "+" }
                        div { class: "select-text flex-1 cursor-pointer", onclick: move |_| manual_collapsed.set(Some(false)),
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str }
                            CollapsedPreview { value: Value::Array(arr) }
                            if !is_last { span { class: "text-gray-400", "," } }
                        }
                    }
                }
            } else {
                let len = arr.len();
                let current_path = path.clone();
                rsx! {
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 h-5 flex-shrink-0 cursor-pointer text-gray-400 hover:text-gray-600 font-bold text-xs inline-flex items-center justify-center", onclick: move |_| manual_collapsed.set(Some(true)), "-" }
                        div { class: "select-text flex-1",
                            NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str: query_str.clone() }
                            span { class: "text-gray-400", "[" }
                        }
                    }
                    { arr.iter().enumerate().map(move |(i, item)| {
                        let child_path = if current_path.is_empty() { format!("/{}", i) } else { format!("{}/{}", current_path, i) };
                        let child_depth = depth + 1;
                        let child_is_last = i == len - 1;
                        let auto_expand = cascade_auto_expand && is_small_container(item);
                        rsx! { JsonNode { key: "{child_path}", value: item.clone(), depth: child_depth, is_last: child_is_last, key_name: None, index: Some(i), path: child_path, auto_expand: auto_expand } }
                    }) }
                    div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                        div { class: "w-4 flex-shrink-0 select-none" }
                        div { class: "select-text flex-1", span { class: "text-gray-400", "]" } if !is_last { span { class: "text-gray-400", "," } } }
                    }
                }
            }
        }
        _ => {
            rsx! {
                div { class: "flex items-start font-mono text-sm leading-relaxed", style: "padding-left: {indent_px}px",
                    div { class: "w-4 flex-shrink-0 select-none" }
                    div { class: "select-text flex-1",
                        NodeHeader { index, key_name, key_match_idx, active_global_idx, query_str: query_str.clone() }
                        PrimitiveValue { value, val_match_idx, active_global_idx, query_str }
                        if !is_last { span { class: "text-gray-400", "," } }
                    }
                }
            }
        }
    }
}

fn is_small_container(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.len() <= AUTO_EXPAND_LIMIT && !map.is_empty(),
        Value::Array(arr) => arr.len() <= AUTO_EXPAND_LIMIT && !arr.is_empty(),
        _ => false,
    }
}

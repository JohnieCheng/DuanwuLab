use serde_json::Value;

use super::types::JsonMatch;

pub(super) fn find_json_matches(
    value: &Value,
    q: &str,
    current_path: &str,
    matches: &mut Vec<JsonMatch>,
) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let next_path = if current_path.is_empty() {
                    format!("/{}", k)
                } else {
                    format!("{}/{}", current_path, k)
                };
                if k.to_lowercase().contains(q) {
                    matches.push(JsonMatch { path: next_path.clone(), is_key: true });
                }
                find_json_matches(v, q, &next_path, matches);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next_path = if current_path.is_empty() {
                    format!("/{}", i)
                } else {
                    format!("{}/{}", current_path, i)
                };
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

use dioxus::prelude::*;

use crate::components::common::clear_button::ClearButton;
use crate::components::common::output_with_copy::OutputWithCopy;

/// JWT decoder — strips Bearer prefix, splits segments, decodes header and payload.
#[component]
pub fn Jwt() -> Element {
    let mut input = use_signal(String::new);
    let mut output = use_signal(String::new);

    let mut decode_jwt = move || {
        let raw = input.read().to_string();
        // Extract JWT from arbitrary text (e.g., curl, Authorization header, cookie)
        let token = extract_jwt(&raw);
        let Some(token) = token else {
            output.set("No valid JWT token found in input.".into());
            return;
        };

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            output.set("Invalid JWT — expected 3 segments separated by '.'".into());
            return;
        }

        let header = decode_segment(parts[0].trim());
        let payload = decode_segment(parts[1].trim());
        let signature = parts[2];

        let mut result = String::new();
        result.push_str("Header:\n");
        result.push_str(&header);
        result.push_str("\n\nPayload:\n");
        result.push_str(&payload);
        result.push_str(&format!("\n\nSignature (raw):\n{}", signature));
        output.set(result);
    };

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400",
                    "JWT Token"
                }
                div { class: "relative rounded-lg border bg-white dark:bg-gray-900 transition-all duration-200 border-gray-200 dark:border-gray-700 focus-within:border-gray-400",
                    textarea {
                        id: "jwt-input",
                        class: "min-h-[120px] w-full resize-y rounded-lg border-0 bg-transparent p-4 font-mono text-sm text-gray-900 focus:outline-none dark:text-gray-100 select-text focus:ring-0",
                        spellcheck: false,
                        placeholder: "eyJhbG... or Bearer eyJhbG...",
                        oninput: move |e: FormEvent| {
                            input.set(e.value());
                            decode_jwt();
                        },
                    }
                    if !input.read().is_empty() {
                        ClearButton {
                            input_id: "jwt-input",
                            on_clear: move |_| { input.set(String::new()); output.set(String::new()); },
                        }
                    }
                }
            }

            if !output.read().is_empty() {
                OutputWithCopy { output: output }
            }
        }
    }
}

fn extract_jwt(raw: &str) -> Option<String> {
    let s = decode_percent(raw).unwrap_or_else(|_| raw.to_string());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Skip non-b64url chars, try to match a JWT from here
        if is_b64url(chars[i]) {
            if let Some(end) = find_jwt_end(&chars, i) {
                let candidate: String = chars[i..end].iter().collect();
                let segments: Vec<&str> = candidate.split('.').collect();
                if segments.len() == 3 && segments.iter().all(|s| s.len() >= 1) {
                    return Some(candidate);
                }
            }
        }
        i += 1;
    }
    None
}

fn find_jwt_end(chars: &[char], start: usize) -> Option<usize> {
    let mut end = start;
    let mut dots = 0;
    while end < chars.len() {
        let c = chars[end];
        if is_b64url(c) {
            end += 1;
        } else if c == '.' && dots < 2 {
            dots += 1;
            end += 1;
        } else {
            // Non-b64url, non-dot (or too many dots) — JWT ends here
            return if dots == 2 && end > start { Some(end) } else { None };
        }
    }
    // End of string — valid only if we saw exactly 2 dots
    if dots == 2 && end > start { Some(end) } else { None }
}

fn is_b64url(c: char) -> bool {
    matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_')
}

fn decode_segment(seg: &str) -> String {
    let padded = pad(seg);
    match base64_decode(&padded) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(s) => {
                let pretty = serde_json::from_str::<serde_json::Value>(&s)
                    .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.clone()))
                    .unwrap_or_else(|_| s);
                pretty
            }
            Err(_) => format!("<binary data, {} bytes>", padded.len() * 3 / 4),
        },
        Err(e) => format!("<decode error: {e}>"),
    }
}

fn decode_percent(s: &str) -> Result<String, ()> {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val(bytes[i + 1])?;
            let lo = hex_val(bytes[i + 2])?;
            out.push(((hi << 4) | lo) as char);
            i += 3;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    Ok(out)
}

fn hex_val(c: u8) -> Result<u8, ()> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(()),
    }
}

fn pad(s: &str) -> String {
    let mut s = s.to_string();
    while s.len() % 4 != 0 {
        s.push('=');
    }
    s
}

fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let input = input.trim_end_matches('=');
    let mut bytes = Vec::with_capacity(input.len() * 3 / 4);
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    for chunk in input.as_bytes().chunks(4) {
        if chunk.is_empty() {
            continue;
        }
        let mut vals = [0u32; 4];
        for (i, &c) in chunk.iter().enumerate() {
            vals[i] = match table.iter().position(|&x| x == c) {
                Some(idx) => idx as u32,
                None => return Err("Invalid base64url character"),
            };
        }
        let n = chunk.len();
        let triple = (vals[0] << 18)
            | (vals.get(1).copied().unwrap_or(0) << 12)
            | (vals.get(2).copied().unwrap_or(0) << 6)
            | vals.get(3).copied().unwrap_or(0);
        bytes.push(((triple >> 16) & 0xFF) as u8);
        if n > 2 {
            bytes.push(((triple >> 8) & 0xFF) as u8);
        }
        if n > 3 {
            bytes.push((triple & 0xFF) as u8);
        }
    }
    Ok(bytes)
}

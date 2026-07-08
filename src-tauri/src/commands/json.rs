use serde_json::Value;

#[tauri::command]
pub fn format_json(input: &str, repair: bool) -> Result<String, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Ok(String::new());
    }
    let (result, repaired) = if repair {
        match parse(raw) {
            Ok(v) => (v, false),
            Err(_) => {
                let fixed = repair_json(raw);
                match parse(&fixed) {
                    Ok(v) => (v, true),
                    Err(e) => {
                        return Err(e.to_string());
                    }
                }
            }
        }
    } else {
        (parse(raw).map_err(|e| e.to_string())?, false)
    };

    let pretty = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
    let note = if repaired { "\n\n// Repaired" } else { "" };
    Ok(format!("{}{}", pretty, note))
}

fn parse(input: &str) -> Result<Value, String> {
    serde_json::from_str(input).map_err(|e| e.to_string())
}

fn repair_json(s: &str) -> String {
    let mut out = s.to_string();

    // 1. trailing commas before '}' or ']'
    out = out.replace(",}", "}").replace(",]", "]");
    out = out.replace(", }", " }").replace(", ]", " ]");
    out = out.replace(",\n}", "\n}").replace(",\n]", "\n]");

    // 2. single quotes → double quotes (outside strings)
    out = out.replace('\'', "\"");

    // 3. unquoted keys: scan for word: pattern
    out = fix_unquoted_keys(&out);

    // 4. missing closing braces
    let (open_c, close_c) = (out.matches('{').count(), out.matches('}').count());
    let (open_s, close_s) = (out.matches('[').count(), out.matches(']').count());
    for _ in 0..open_c.saturating_sub(close_c) {
        out.push('}');
    }
    for _ in 0..open_s.saturating_sub(close_s) {
        out.push(']');
    }

    out
}

fn fix_unquoted_keys(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 64);
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // skip existing strings
        if c == '"' {
            result.push(c);
            i += 1;
            while i < len {
                if chars[i] == '\\' {
                    result.push(chars[i]);
                    i += 1;
                    if i < len {
                        result.push(chars[i]);
                    }
                } else if chars[i] == '"' {
                    result.push(chars[i]);
                    i += 1;
                    break;
                } else {
                    result.push(chars[i]);
                }
                i += 1;
            }
            continue;
        }

        // skip whitespace/separators
        if c != '_' && !c.is_alphabetic() {
            result.push(c);
            i += 1;
            continue;
        }

        // potential unquoted key: capture the word
        let start = i;
        while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        let word: String = chars[start..i].iter().collect();

        // skip whitespace
        let ws_start = i;
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // check if followed by ':'
        if i < len && chars[i] == ':' {
            result.push('"');
            result.push_str(&word);
            result.push('"');
            // restore whitespace
            result.extend(chars[ws_start..i].iter().copied());
            result.push(':');
            i += 1;
        } else {
            // not a key, restore word and whitespace
            result.push_str(&word);
            result.extend(chars[ws_start..i].iter().copied());
        }
    }

    result
}

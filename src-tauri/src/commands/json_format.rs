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
    let mut out = tokenize_and_repair(s);

    // Balance braces/brackets
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

/// Single-pass tokenizer that repairs common JSON issues in one scan:
/// - Fixes invalid escape sequences inside strings (doubles lone backslashes)
/// - Converts single-quoted strings to double-quoted
/// - Removes trailing commas before '}' or ']'
/// - Quotes unquoted keys (bare words followed by `:`)
///
/// This is designed for log data where string values often contain
/// literal backslashes (Windows paths, stack traces, etc.) that are
/// invalid in JSON.
fn tokenize_and_repair(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 256);
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // ── Double-quoted string ──
        if c == '"' {
            result.push('"');
            i += 1;

            while i < len {
                match chars[i] {
                    '\\' => {
                        i += 1;
                        if i < len {
                            let next = chars[i];
                            // Structural escapes that must stay as escapes
                            if next == '"' || next == '\\' || next == '/' {
                                result.push('\\');
                                result.push(next);
                            } else if next == 'u' {
                                // Unicode escape: \uXXXX — pass through
                                result.push('\\');
                                result.push('u');
                                i += 1;
                                for _ in 0..4 {
                                    if i < len && chars[i].is_ascii_hexdigit() {
                                        result.push(chars[i]);
                                        i += 1;
                                    } else {
                                        break;
                                    }
                                }
                                continue; // skip the i += 1 at end of loop
                            } else {
                                // Invalid JSON escape (\b \f \n \r \t or totally invalid like \p \U \w).
                                // In repair mode we treat the backslash as literal by doubling it.
                                // This preserves log content (paths, stack traces) at the cost of
                                // losing escape semantics (newlines, tabs become literal \n \t).
                                result.push('\\');
                                result.push('\\');
                                result.push(next);
                            }
                        } else {
                            // Trailing backslash at end-of-input
                            result.push('\\');
                            result.push('\\');
                        }
                    }
                    '"' => {
                        result.push('"');
                        i += 1;
                        break;
                    }
                    other => {
                        result.push(other);
                    }
                }
                i += 1;
            }
            continue;
        }

        // ── Single-quoted string → convert to double-quoted ──
        if c == '\'' {
            result.push('"');
            i += 1;

            while i < len {
                match chars[i] {
                    '\\' => {
                        i += 1;
                        if i < len {
                            match chars[i] {
                                '\'' => {
                                    // Escaped single-quote → literal single-quote
                                    // (no escaping needed inside double-quoted JSON)
                                    result.push('\'');
                                }
                                '\\' => {
                                    result.push('\\');
                                    result.push('\\');
                                }
                                '"' => {
                                    // Already-escaped double-quote inside single-quoted string
                                    result.push('\\');
                                    result.push('"');
                                }
                                other => {
                                    // Preserve other escapes (\n, \t, etc.) as-is
                                    result.push('\\');
                                    result.push(other);
                                }
                            }
                        } else {
                            result.push('\\');
                            result.push('\\');
                        }
                    }
                    '\'' => {
                        result.push('"');
                        i += 1;
                        break;
                    }
                    '"' => {
                        // Bare double-quote inside single-quoted string: escape it
                        result.push('\\');
                        result.push('"');
                    }
                    other => {
                        result.push(other);
                    }
                }
                i += 1;
            }
            continue;
        }

        // ── Trailing comma before '}' or ']' ──
        if c == ',' {
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && (chars[j] == '}' || chars[j] == ']') {
                // Skip the comma; preserve whitespace after it
                i += 1;
                while i < j {
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
            result.push(',');
            i += 1;
            continue;
        }

        // ── Unquoted key: bare word followed by optional whitespace + ':' ──
        if c == '_' || c == '$' || c.is_alphabetic() {
            let word_start = i;
            while i < len
                && (chars[i].is_alphanumeric()
                    || chars[i] == '_'
                    || chars[i] == '$'
                    || chars[i] == '-')
            {
                i += 1;
            }
            let word: String = chars[word_start..i].iter().collect();

            // Consume whitespace after the word
            let ws_start = i;
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            if i < len && chars[i] == ':' {
                // Confirmed: it's an unquoted key
                result.push('"');
                result.push_str(&word);
                result.push('"');
                // Restore whitespace
                for k in ws_start..i {
                    if chars[k].is_whitespace() {
                        result.push(chars[k]);
                    }
                }
                result.push(':');
                i += 1;
            } else {
                // False alarm: restore word + whitespace unchanged
                result.push_str(&word);
                for k in ws_start..i {
                    result.push(chars[k]);
                }
            }
            continue;
        }

        // ── Pass-through everything else ──
        result.push(c);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_path_backslashes() {
        let input = r#"{"file": "C:\Users\johnie\Documents"}"#;
        let result = repair_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["file"], "C:\\Users\\johnie\\Documents");
    }

    #[test]
    fn test_unc_path_backslashes() {
        // UNC path \\host\share — \s is invalid, should be fixed
        let input = r#"{"path": "\\host\share"}"#;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    #[test]
    fn test_double_escaped_json_in_string() {
        // String value contains escaped JSON
        let input = r#"{"payload": "{\"key\":\"value\"}"}"#;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    #[test]
    fn test_triple_backslash_quote() {
        // The \\\ " pattern: escaped backslash + escaped quote (common in log payloads)
        let input = r#"{"data": "{\\\"key\\\":\\\"value\\\"}"}"#;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    #[test]
    fn test_single_quotes_as_delimiters() {
        let input = "{'name':'john','msg':'it\\'s ok'}";
        let result = repair_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "john");
        assert_eq!(parsed["msg"], "it's ok");
    }

    #[test]
    fn test_trailing_comma() {
        let input = r#"{"a": 1,}"#;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    #[test]
    fn test_unquoted_keys() {
        let input = r#"{name: "john", age: 30}"#;
        let result = repair_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "john");
        assert_eq!(parsed["age"], 30);
    }

    #[test]
    fn test_mixed_escapes_and_paths() {
        // Log message with Windows path + properly escaped quotes
        let input = r#"{"msg": "Error at C:\path\to\file - user=\"john\""}"#;
        let result = repair_json(input);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let msg = parsed["msg"].as_str().unwrap();
        assert!(msg.contains("C:\\path\\to\\file"));
        assert!(msg.contains("user=\"john\""));
    }

    #[test]
    fn test_missing_closing_braces() {
        let input = r#"{"a": {"b": [1, 2]"#;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    #[test]
    fn test_real_world_log_payload() {
        // Input: {"payload":"{\"content\":\"{\\\"sCode\\\":\\\"FP001\\\"}\"}"}
        // The payload string value is: {"content":"{\"sCode\":\"FP001\"}"}
        // This is valid, properly-escaped JSON with deep nesting.
        let input = r##"{"payload":"{\"content\":\"{\\\"sCode\\\":\\\"FP001\\\"}\"}"}"##;
        let result = repair_json(input);
        assert!(serde_json::from_str::<Value>(&result).is_ok(), "Repaired JSON should be valid");
        // Verify the nested structure survives
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let payload_str = parsed["payload"].as_str().unwrap();
        let inner: Value = serde_json::from_str(payload_str).unwrap();
        let content_str = inner["content"].as_str().unwrap();
        let innermost: Value = serde_json::from_str(content_str).unwrap();
        assert_eq!(innermost["sCode"], "FP001");
    }
}

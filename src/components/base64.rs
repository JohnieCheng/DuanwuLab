use dioxus::document::eval;
use dioxus::prelude::*;

use crate::components::common::output_with_copy::OutputWithCopy;

/// Standalone Base64 encode / decode tool.
#[component]
pub fn Base64() -> Element {
    let mut input = use_signal(String::new);
    let mut output = use_signal(String::new);

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            div { class: "flex flex-col gap-2",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 dark:text-gray-400",
                    "Input"
                }
                div { class: "relative rounded-lg border bg-white dark:bg-gray-900 transition-all duration-200 border-gray-200 dark:border-gray-700 focus-within:border-gray-400",
                    textarea {
                        id: "base64-input",
                        class: "min-h-[150px] w-full resize-y rounded-lg border-0 bg-transparent p-4 font-mono text-sm text-gray-900 focus:outline-none dark:text-gray-100 select-text focus:ring-0",
                        spellcheck: false,
                        placeholder: "Text to encode / decode ...",
                        oninput: move |e: FormEvent| input.set(e.value()),
                    }
                    if !input.read().is_empty() {
                        button {
                            class: "absolute top-3 right-3 rounded border border-gray-200 bg-white px-2 py-0.5 text-[11px] text-gray-400 hover:text-red-500 hover:border-red-200 dark:border-gray-700 dark:bg-gray-900 dark:hover:text-red-400 transition-colors select-none",
                            onclick: move |_| {
                                input.set(String::new());
                                eval("var t=document.getElementById('base64-input');if(t)t.value=''");
                            },
                            "✕ Clear"
                        }
                    }
                }
            }

            div { class: "flex items-center gap-3",
                button {
                    class: "rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white dark:bg-white dark:text-gray-900",
                    onclick: move |_| output.set(base64_encode(&input.read())),
                    "Encode"
                }
                button {
                    class: "rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm font-medium text-gray-700 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300",
                    onclick: move |_| {
                        match base64_decode(&input.read()) {
                            Ok(s) => output.set(s),
                            Err(e) => output.set(format!("Decode error: {e}")),
                        }
                    },
                    "Decode"
                }
            }

            if !output.read().is_empty() {
                OutputWithCopy { output: output }
            }
        }
    }
}

// ── Pure-Rust base64 (no external crate) ──

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        result.push(if chunk.len() > 1 {
            CHARS[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 { CHARS[(triple & 0x3F) as usize] as char } else { '=' });
    }

    result
}

fn base64_decode(input: &str) -> Result<String, &'static str> {
    let input = input.trim_end_matches('=');
    let mut bytes = Vec::with_capacity(input.len() * 3 / 4);

    for chunk in input.as_bytes().chunks(4) {
        if chunk.len() < 2 {
            return Err("Invalid base64 length");
        }

        let mut sextets = [0u32; 4];
        for (i, &c) in chunk.iter().enumerate() {
            sextets[i] = match c {
                b'A'..=b'Z' => (c - b'A') as u32,
                b'a'..=b'z' => (c - b'a' + 26) as u32,
                b'0'..=b'9' => (c - b'0' + 52) as u32,
                b'+' => 62,
                b'/' => 63,
                _ => return Err("Invalid base64 character"),
            };
        }

        let triple = (sextets[0] << 18)
            | (sextets[1] << 12)
            | if chunk.len() > 2 { sextets[2] << 6 } else { 0 }
            | if chunk.len() > 3 { sextets[3] } else { 0 };

        bytes.push(((triple >> 16) & 0xFF) as u8);
        if chunk.len() > 2 {
            bytes.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk.len() > 3 {
            bytes.push((triple & 0xFF) as u8);
        }
    }

    String::from_utf8(bytes).map_err(|_| "Invalid UTF-8 in decoded data")
}

use dioxus::document::eval;
use dioxus::prelude::*;

use crate::components::common::output_with_copy::OutputWithCopy;

const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// Standalone URL encode / decode tool.
#[component]
pub fn UrlCodec() -> Element {
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
                        id: "url-input",
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
                                eval("var t=document.getElementById('url-input');if(t)t.value=''");
                            },
                            "✕ Clear"
                        }
                    }
                }
            }

            div { class: "flex items-center gap-3",
                button {
                    class: "rounded-lg bg-gray-900 px-4 py-2 text-sm font-medium text-white dark:bg-white dark:text-gray-900",
                    onclick: move |_| output.set(url_encode(&input.read())),
                    "Encode URL"
                }
                button {
                    class: "rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm font-medium text-gray-700 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300",
                    onclick: move |_| output.set(url_encode_component(&input.read())),
                    "Encode Component"
                }
                button {
                    class: "rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm font-medium text-gray-700 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-300",
                    onclick: move |_| {
                        match url_decode(&input.read()) {
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

// ── Pure-Rust URL encode / decode ──

fn url_encode(s: &str) -> String {
    url_encode_inner(s, false)
}

fn url_encode_component(s: &str) -> String {
    url_encode_inner(s, true)
}

fn url_encode_inner(s: &str, component: bool) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for &byte in s.as_bytes() {
        let keep = match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => true,
            // encodeURI keeps ; , / ? : @ & = + $ #
            b';' | b',' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b'#' => {
                !component
            }
            _ => false,
        };
        if keep {
            result.push(byte as char);
        } else {
            result.push('%');
            result.push(HEX[((byte >> 4) & 0xF) as usize] as char);
            result.push(HEX[(byte & 0xF) as usize] as char);
        }
    }
    result
}

fn url_decode(s: &str) -> Result<String, &'static str> {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err("Truncated percent-escape");
                }
                let hi = hex_val(bytes[i + 1])?;
                let lo = hex_val(bytes[i + 2])?;
                result.push((hi << 4) | lo);
                i += 3;
            }
            b'+' => {
                result.push(b' ');
                i += 1;
            }
            b => {
                result.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(result).map_err(|_| "Invalid UTF-8 in decoded data")
}

fn hex_val(c: u8) -> Result<u8, &'static str> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err("Invalid hex digit in percent-escape"),
    }
}

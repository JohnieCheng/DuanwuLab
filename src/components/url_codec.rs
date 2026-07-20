use dioxus::prelude::*;

use crate::components::common::button::{Button, ButtonVariant};
use crate::components::common::clear_button::ClearButton;
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
                        ClearButton { input_id: "url-input", on_clear: move |_| { input.set(String::new()); output.set(String::new()); } }
                    }
                }
            }

            div { class: "flex items-center gap-3",
                Button { label: "Encode URL", variant: ButtonVariant::Solid, onclick: move |_| output.set(url_encode(&input.read())) }
                Button { label: "Encode Component", variant: ButtonVariant::Outline, onclick: move |_| output.set(url_encode_component(&input.read())) }
                Button { label: "Decode", variant: ButtonVariant::Outline, onclick: move |_| {
                    match url_decode(&input.read()) {
                        Ok(s) => output.set(s),
                        Err(e) => output.set(format!("Decode error: {e}")),
                    }
                }}
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

use dioxus::prelude::*;

use crate::components::common::button::{Button, ButtonVariant};
use crate::components::common::output_with_copy::OutputWithCopy;
use crate::components::common::pill::Pill;

/// Password generator with configurable strength and batch support.
#[component]
pub fn PasswordGen() -> Element {
    let default_len = 16u16;
    let default_count = 5u16;

    let mut length = use_signal(|| default_len);
    let mut upper = use_signal(|| true);
    let mut lower = use_signal(|| true);
    let mut digits = use_signal(|| true);
    let mut symbols = use_signal(|| false);
    let mut count = use_signal(|| default_count);
    let mut refresh_trigger = use_signal(|| 0u32);

    let passwords = use_memo(move || {
        let _ = *refresh_trigger.read();
        let is_upper = *upper.read();
        let is_lower = *lower.read();
        let is_digits = *digits.read();
        let is_symbols = *symbols.read();
        let len = *length.read();
        let cnt = *count.read();
        let charset = build_charset(is_upper, is_lower, is_digits, is_symbols);
        if charset.is_empty() {
            return String::from("Select at least one character set.");
        }
        let safe_length = len.clamp(4, 128) as u8;
        let safe_count = cnt.clamp(1, 50) as u8;
        generate_passwords_csprng(&charset, safe_length, safe_count)
            .unwrap_or_else(|_| String::from("Error: Failed to gather secure system entropy."))
    });

    let p: ReadSignal<String> = passwords.into();

    let current_length = *length.read();
    let current_count = *count.read();
    let is_upper = *upper.read();
    let is_lower = *lower.read();
    let is_digits = *digits.read();
    let is_symbols = *symbols.read();

    let reset_to_defaults = move |_| {
        length.set(default_len);
        upper.set(true);
        lower.set(true);
        digits.set(true);
        symbols.set(false);
        count.set(default_count);
    };

    let input_style = "w-14 rounded-lg border border-gray-200 bg-white px-1 py-1 text-sm \
                       text-gray-700 text-center transition-colors focus:outline-none \
                       focus:border-gray-400 dark:border-gray-700 dark:bg-gray-900 \
                       dark:text-gray-300 [appearance:textfield] \
                       [&::-webkit-outer-spin-button]:appearance-none \
                       [&::-webkit-inner-spin-button]:appearance-none";

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",
            div { class: "flex flex-wrap items-center gap-6",
                div { class: "flex items-center gap-3",
                    span { class: "text-xs font-medium text-gray-400 dark:text-gray-500", "Length" }
                    input {
                        class: "w-24 h-1 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-gray-800 dark:accent-gray-200",
                        r#type: "range",
                        min: 4,
                        max: 128,
                        value: "{current_length}",
                        oninput: move |e: FormEvent| {
                            if let Ok(v) = e.value().parse::<u16>() {
                                let current = *length.read();
                                if current != v { length.set(v); }
                            }
                        },
                    }
                    input {
                        class: "{input_style}",
                        r#type: "number",
                        placeholder: "16",
                        value: "{current_length}",
                        oninput: move |e: FormEvent| {
                            let val = e.value();
                            if val.is_empty() {
                                if *length.read() != 0 { length.set(0); }
                            } else {
                                match val.parse::<u32>() {
                                    Ok(v) => {
                                        let n = v.min(999) as u16;
                                        if *length.read() != n { length.set(n); }
                                    }
                                    Err(_) => if val.chars().all(|c| c.is_ascii_digit()) && *length.read() != 999 { length.set(999); }
                                }
                            }
                        },
                        onblur: move |_| {
                            let c = *length.read();
                            let clamped = c.clamp(4, 128);
                            if c != clamped { length.set(clamped); }
                        },
                    }
                }
                div { class: "flex items-center gap-2",
                    span { class: "text-xs font-medium text-gray-400 dark:text-gray-500", "Batch" }
                    input {
                        class: "{input_style}",
                        r#type: "number",
                        placeholder: "5",
                        value: "{current_count}",
                        oninput: move |e: FormEvent| {
                            let val = e.value();
                            if val.is_empty() {
                                if *count.read() != 0 { count.set(0); }
                            } else {
                                match val.parse::<u32>() {
                                    Ok(v) => {
                                        let n = v.min(999) as u16;
                                        if *count.read() != n { count.set(n); }
                                    }
                                    Err(_) => if val.chars().all(|c| c.is_ascii_digit()) && *count.read() != 999 { count.set(999); }
                                }
                            }
                        },
                        onblur: move |_| {
                            let c = *count.read();
                            let clamped = c.clamp(1, 50);
                            if c != clamped { count.set(clamped); }
                        },
                    }
                }
            }
            div { class: "flex flex-wrap items-center justify-between gap-4 pb-2",
                div { class: "flex flex-wrap items-center gap-3",
                    Pill { label: "A–Z", active: is_upper, onclick: move |_| upper.set(!is_upper) }
                    Pill { label: "a–z", active: is_lower, onclick: move |_| lower.set(!is_lower) }
                    Pill { label: "0–9", active: is_digits, onclick: move |_| digits.set(!is_digits) }
                    Pill { label: "!@#", active: is_symbols, onclick: move |_| symbols.set(!is_symbols) }
                }
                div { class: "flex items-center gap-2.5",
                    Button { label: "Reset", variant: ButtonVariant::Subtle, onclick: reset_to_defaults }
                    Button { label: "Regen", variant: ButtonVariant::Subtle, onclick: move |_| *refresh_trigger.write() += 1 }
                }
            }
            div { class: "h-px w-full bg-gray-100/80 dark:bg-gray-800/40 my-2" }
            if !passwords.read().is_empty() {
                OutputWithCopy { output: p }
            }
        }
    }
}

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>?";

fn build_charset(upper: bool, lower: bool, digits: bool, symbols: bool) -> Vec<u8> {
    let mut cs = Vec::with_capacity(95);
    if upper {
        cs.extend_from_slice(UPPER);
    }
    if lower {
        cs.extend_from_slice(LOWER);
    }
    if digits {
        cs.extend_from_slice(DIGITS);
    }
    if symbols {
        cs.extend_from_slice(SYMBOLS);
    }
    cs
}

fn generate_passwords_csprng(
    charset: &[u8],
    length: u8,
    count: u8,
) -> Result<String, getrandom::Error> {
    let len = length as usize;
    let total = count as usize;
    let charset_len = charset.len();
    let mut out = String::with_capacity(total * len + total.saturating_sub(1));
    let limit = 256 - (256 % charset_len);
    let mut entropy_buffer = [0u8; 256];
    let entropy_slice = &mut entropy_buffer[..len * 2];
    for i in 0..total {
        if i > 0 {
            out.push('\n');
        }
        let mut filled = 0;
        while filled < len {
            getrandom::getrandom(entropy_slice)?;
            for &byte in entropy_slice.iter() {
                let val = byte as usize;
                if val < limit {
                    out.push(charset[val % charset_len] as char);
                    filled += 1;
                    if filled == len {
                        break;
                    }
                }
            }
        }
    }
    Ok(out)
}

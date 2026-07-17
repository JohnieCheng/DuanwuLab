use dioxus::prelude::*;

use crate::components::common::output_with_copy::OutputWithCopy;

/// 密码生成器主组件：支持强度配置、批量并发生成、密码学安全、一键重置与手动刷新
#[component]
pub fn PasswordGen() -> Element {
    // 默认配置常量
    let default_len = 16u16;
    let default_count = 5u16;

    let mut length = use_signal(|| default_len);
    let mut upper = use_signal(|| true);
    let mut lower = use_signal(|| true);
    let mut digits = use_signal(|| true);
    let mut symbols = use_signal(|| false);
    let mut count = use_signal(|| default_count);

    // 用于手动触发“重新生成”的依赖信号
    let mut refresh_trigger = use_signal(|| 0u32);

    // 1. 记忆化密码生成逻辑 (Memoization)
    let passwords = use_memo(move || {
        // 读取 refresh_trigger 以将其绑定为依赖，每次 trigger + 1 都会重新计算
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

        // 💡 优雅降级：用惰性求值的 unwrap_or_else 代替 match，无额外堆分配
        generate_passwords_csprng(&charset, safe_length, safe_count)
            .unwrap_or_else(|_| String::from("Error: Failed to gather secure system entropy."))
    });

    let p: ReadSignal<String> = passwords.into();

    // 2. 状态快照化 (Snapshotting) - 防止 DOM 渲染树中产生不必要的锁竞争
    let current_length = *length.read();
    let current_count = *count.read();
    let is_upper = *upper.read();
    let is_lower = *lower.read();
    let is_digits = *digits.read();
    let is_symbols = *symbols.read();

    // 一键重置闭包
    let reset_to_defaults = move |_| {
        length.set(default_len);
        upper.set(true);
        lower.set(true);
        digits.set(true);
        symbols.set(false);
        count.set(default_count);
    };

    // Tailwind 样式抽取
    let input_style = "w-14 rounded-lg border border-gray-200 bg-white px-1 py-1 text-sm \
                       text-gray-700 text-center transition-colors focus:outline-none \
                       focus:border-gray-400 dark:border-gray-700 dark:bg-gray-900 \
                       dark:text-gray-300 [appearance:textfield] \
                       [&::-webkit-outer-spin-button]:appearance-none \
                       [&::-webkit-inner-spin-button]:appearance-none";

    let action_btn_style = "flex items-center gap-1.5 px-3 py-1 text-xs font-medium \
                            text-gray-500 hover:text-gray-800 bg-gray-50/30 hover:bg-gray-100/80 \
                            border border-gray-100 hover:border-gray-300 rounded-full \
                            transition-all duration-200 cursor-pointer \
                            dark:text-gray-400 dark:hover:text-gray-200 dark:bg-gray-900/30 \
                            dark:hover:bg-gray-800/80 dark:border-gray-800 dark:hover:border-gray-600 \
                            select-none focus:outline-none";

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 p-6 select-none",

            // 1. 控件控制区
            div { class: "flex flex-wrap items-center gap-6",

                // 长度控制：Slider 滑块 + Number 精确数字框
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
                                if current != v {
                                    length.set(v);
                                }
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
                                let current = *length.read();
                                if current != 0 {
                                    length.set(0);
                                }
                            } else {
                                match val.parse::<u32>() {
                                    Ok(v) => {
                                        let new_val = v.min(999) as u16;
                                        let current = *length.read();
                                        if current != new_val {
                                            length.set(new_val);
                                        }
                                    }
                                    Err(_) => {
                                        if val.chars().all(|c| c.is_ascii_digit()) {
                                            if *length.read() != 999 {
                                                length.set(999);
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        onblur: move |_| {
                            let current = *length.read();
                            let clamped = current.clamp(4, 128);
                            if current != clamped {
                                length.set(clamped);
                            }
                        }
                    }
                }

                // 批量生成数控制
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
                                let current = *count.read();
                                if current != 0 {
                                    count.set(0);
                                }
                            } else {
                                match val.parse::<u32>() {
                                    Ok(v) => {
                                        let new_val = v.min(999) as u16;
                                        let current = *count.read();
                                        if current != new_val {
                                            count.set(new_val);
                                        }
                                    }
                                    Err(_) => {
                                        if val.chars().all(|c| c.is_ascii_digit()) {
                                            if *count.read() != 999 {
                                                count.set(999);
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        onblur: move |_| {
                            let current = *count.read();
                            let clamped = current.clamp(1, 50);
                            if current != clamped {
                                count.set(clamped);
                            }
                        }
                    }
                }
            }

            // 2. 字符集过滤按钮与功能操作
            div { class: "flex flex-wrap items-center justify-between gap-4 pb-2",

                // 字符开关组
                div { class: "flex flex-wrap items-center gap-3",
                    Toggle { label: "A–Z", active: is_upper, onclick: move |_| upper.set(!is_upper) }
                    Toggle { label: "a–z", active: is_lower, onclick: move |_| lower.set(!is_lower) }
                    Toggle { label: "0–9", active: is_digits, onclick: move |_| digits.set(!is_digits) }
                    Toggle { label: "!@#", active: is_symbols, onclick: move |_| symbols.set(!is_symbols) }
                }

                // 操作功能按钮组（重置、重新生成）
                div { class: "flex items-center gap-2.5",

                    // 重置按钮
                    button {
                        r#type: "button",
                        class: "{action_btn_style}",
                        onclick: reset_to_defaults,
                        span { "Reset" }
                    }

                    // 手动刷新生成按钮（* 显式解引用修复 E0368）
                    button {
                        r#type: "button",
                        class: "{action_btn_style}",
                        onclick: move |_| *refresh_trigger.write() += 1,

                        // 细线版 Refresh 经典图标
                        svg {
                            class: "w-3.5 h-3.5",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99"
                            }
                        }
                        span { "Regen" }
                    }
                }
            }

            // 💡 替代 border-b 的极细拟物分割线：避免黑边框渲染 Bug，完美融合明暗主题
            div { class: "h-px w-full bg-gray-100/80 dark:bg-gray-800/40 my-2" }

            // 3. 输出显示组件
            if !passwords.read().is_empty() {
                OutputWithCopy { output: p }
            }
        }
    }
}

/// 药丸形状的无障碍切换按钮
#[component]
fn Toggle(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let base = "rounded-full px-3 py-1 text-xs font-medium transition-colors cursor-pointer select-none \
                focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-gray-300 dark:focus:ring-gray-700";
    let state = if active {
        "bg-gray-800 text-white dark:bg-gray-200 dark:text-gray-900"
    } else {
        "bg-gray-100 text-gray-500 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700"
    };
    rsx! {
        button {
            r#type: "button",
            class: "{base} {state}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

// --------------------------- 核心安全生成算法 ---------------------------

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>?";

fn build_charset(upper: bool, lower: bool, digits: bool, symbols: bool) -> Vec<u8> {
    let mut cs = Vec::with_capacity(95);
    if upper { cs.extend_from_slice(UPPER); }
    if lower { cs.extend_from_slice(LOWER); }
    if digits { cs.extend_from_slice(DIGITS); }
    if symbols { cs.extend_from_slice(SYMBOLS); }
    cs
}

/// 🚀 极致性能优化版：零多余堆分配 + 栈缓冲区 + 消除模偏差
fn generate_passwords_csprng(charset: &[u8], length: u8, count: u8) -> Result<String, getrandom::Error> {
    let len = length as usize;
    let total = count as usize;
    let charset_len = charset.len();

    // 1. 极致堆内存优化：一次性预分配整张输出表所需的全部内存，避免循环中产生碎片化的小额堆分配
    let mut out = String::with_capacity(total * len + total.saturating_sub(1));

    let limit = 256 - (256 % charset_len);

    // 2. 极致栈内存优化：因为 safe_length 被限制在 <= 128 范围内，其 2 倍缓冲区最大仅为 256 字节
    // 我们在栈上直接开辟 256 字节数组，彻底省去 Vec 的堆内存申请与释放开销！
    let mut entropy_buffer = [0u8; 256];
    let required_slice_len = len * 2;
    let entropy_slice = &mut entropy_buffer[..required_slice_len];

    for i in 0..total {
        if i > 0 {
            out.push('\n');
        }

        let mut filled = 0;
        while filled < len {
            // 一次性获取批量安全熵源
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
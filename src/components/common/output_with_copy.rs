use dioxus::document::eval;
use dioxus::prelude::*;

/// Reusable output panel with a copy-to-clipboard button.
#[component]
pub fn OutputWithCopy(output: ReadSignal<String>) -> Element {
    let mut copied = use_signal(|| false);

    rsx! {
        div { class: "flex flex-col gap-2",
            div { class: "flex h-8 items-center justify-between",
                label { class: "text-xs font-medium uppercase tracking-wider text-gray-500 px-1",
                    "Output"
                }
                button {
                    class: "rounded border border-gray-200 dark:border-gray-700 px-2 py-0.5 text-[11px] transition-all duration-200 select-none",
                    class: if *copied.read() {
                        "text-emerald-600 border-emerald-200 dark:text-emerald-400 dark:border-emerald-800"
                    } else {
                        "text-gray-400 hover:text-gray-600 hover:border-gray-300 dark:text-gray-500 dark:hover:text-gray-300 bg-white dark:bg-gray-950"
                    },
                    onclick: move |_| {
                        let escaped = serde_json::to_string(&*output.read()).unwrap();
                        eval(&format!("navigator.clipboard.writeText({escaped})"));
                        copied.set(true);
                        spawn(async move {
                            gloo_timers::future::sleep(std::time::Duration::from_millis(1500))
                                .await;
                            copied.set(false);
                        });
                    },
                    if *copied.read() { "✓ Copied" } else { "Copy" }
                }
            }
            pre { class: "whitespace-pre-wrap break-all rounded-lg border border-gray-200 bg-gray-50 p-4 font-mono text-sm text-gray-900 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 select-text",
                "{output}"
            }
        }
    }
}

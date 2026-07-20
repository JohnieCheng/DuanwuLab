use dioxus::prelude::*;

use super::constants::AUTO_FORMAT_DEBOUNCE_MS;
use crate::components::common::clear_button::ClearButton;

#[component]
pub(super) fn JsonInputEditor(
    input_text: Signal<String>,
    on_auto_format: EventHandler<String>,
    on_clear: EventHandler<()>,
) -> Element {
    let mut local_input = use_signal(String::new);

    use_effect(move || {
        let text = local_input.read().clone();
        if text.trim().is_empty() {
            return;
        }
        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(AUTO_FORMAT_DEBOUNCE_MS))
                .await;
            if *local_input.read() == text {
                on_auto_format.call(text);
            }
        });
    });

    rsx! {
        div { class: "relative rounded-lg border bg-white dark:bg-gray-900 transition-all duration-200 border-gray-200 dark:border-gray-700 focus-within:border-gray-400",
            textarea {
                id: "json-input",
                class: "min-h-[200px] w-full resize-y rounded-lg border-0 bg-transparent p-4 font-mono text-sm text-gray-900 focus:outline-none dark:text-gray-100 select-text focus:ring-0",
                spellcheck: false,
                placeholder: "Paste JSON here...",
                oninput: move |e: FormEvent| {
                    let val = e.value();
                    local_input.set(val.clone());
                    input_text.set(val);
                },
            }
            if !local_input.read().is_empty() {
                ClearButton {
                    input_id: "json-input",
                    on_clear: move |_| {
                        local_input.set(String::new());
                        input_text.set(String::new());
                        on_clear.call(());
                    },
                }
            }
        }
    }
}

use dioxus::prelude::*;

/// Pill-shaped dropdown.
#[component]
pub fn PillSelect(
    label: &'static str,
    options: &'static [(&'static str, &'static str)],
    selected: String,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-3",
            span { class: "text-xs text-gray-400 dark:text-gray-500 select-none", "{label}" }
            select {
                class: "appearance-none rounded-full border border-gray-200 bg-transparent py-1.5 pl-3 pr-7 text-xs text-gray-500 bg-no-repeat transition-colors hover:border-gray-300 dark:border-gray-700 dark:text-gray-400 dark:hover:border-gray-600 focus:outline-none",
                style: "background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 10 10'%3E%3Cpath fill='%239ca3af' d='M2 3.5l3 3 3-3'/%3E%3C/svg%3E\");background-position:right 8px center",
                onchange: move |e: FormEvent| onchange.call(e.value()),
                for (value, label) in options.iter() {
                    option { value: *value, selected: selected == *value, "{label}" }
                }
            }
        }
    }
}

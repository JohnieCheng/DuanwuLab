use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Solid,
    Outline,
    Subtle,
    Ghost,
    Danger,
}

/// Pill-shaped button.
#[component]
pub fn Button(
    label: String,
    onclick: EventHandler<MouseEvent>,
    variant: ButtonVariant,
    #[props(default)] id: Option<&'static str>,
    #[props(default = false)] disabled: bool,
) -> Element {
    let (style, base) = match variant {
        ButtonVariant::Solid => (
            "bg-gray-800 text-white hover:opacity-80 dark:bg-gray-200 dark:text-gray-900 \
             dark:hover:opacity-80 transition-colors",
            "rounded-full px-4 py-1.5 text-xs font-medium",
        ),
        ButtonVariant::Outline => (
            "border border-gray-200 text-gray-500 hover:bg-gray-50 dark:border-gray-700 \
             dark:text-gray-400 dark:hover:bg-gray-800",
            "rounded-full px-4 py-1.5 text-xs font-medium",
        ),
        ButtonVariant::Subtle => (
            "text-gray-500 hover:text-gray-800 bg-gray-50/30 hover:bg-gray-100/80 border \
             border-gray-100 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-200 \
             dark:bg-gray-900/30 dark:hover:bg-gray-800/80 dark:border-gray-800 \
             dark:hover:border-gray-600",
            "rounded-full px-4 py-1.5 text-xs font-medium",
        ),
        ButtonVariant::Ghost => ("text-xs text-gray-400 hover:text-gray-500", ""),
        ButtonVariant::Danger => ("text-xs text-gray-400 hover:text-red-500", ""),
    };
    rsx! {
        button {
            r#type: "button",
            class: "{base} {style} transition-colors cursor-pointer select-none disabled:opacity-30",
            id: id,
            disabled: disabled,
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

use dioxus::document::eval;
use dioxus::prelude::*;

use crate::components::error_toast::ErrorToast;
use crate::components::layout::{Page, SidebarLayout};
use crate::context::error::GlobalErrorContext;

static CSS: Asset = asset!("/assets/styles.css");

/// Root component: wires up global error context, page state, stylesheet, and layout.
pub fn App() -> Element {
    let error_message = use_signal(|| None::<String>);
    use_context_provider(move || GlobalErrorContext { message: error_message });

    let page = use_signal(|| Page::JsonFormatter);

    // Global keyboard shortcuts: block Ctrl+A everywhere except in inputs/textareas
    use_effect(move || {
        let js_code = r#"
            (function() {
                if (window._appKeyHandler) { return; }
                window._appKeyHandler = function(e) {
                    if ((e.ctrlKey || e.metaKey) && e.code === 'KeyA') {
                        const active = document.activeElement;
                        if (!active || (active.tagName !== 'INPUT' && active.tagName !== 'TEXTAREA')) {
                            e.preventDefault();
                        }
                    }
                };
                window.addEventListener('keydown', window._appKeyHandler);
            })();
        "#;
        eval(js_code);
    });

    rsx! {
        ErrorToast { message: error_message }
        link { rel: "stylesheet", href: CSS }
        SidebarLayout { page }
    }
}

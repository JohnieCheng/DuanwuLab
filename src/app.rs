use dioxus::prelude::*;

use crate::components::error_toast::ErrorToast;
use crate::components::layout::SidebarLayout;
use crate::context::error::GlobalErrorContext;

static CSS: Asset = asset!("/assets/styles.css");

/// Root component: wires up global error context, stylesheet, and layout.
pub fn App() -> Element {
    let error_message = use_signal(|| None::<String>);
    use_context_provider(move || GlobalErrorContext { message: error_message });

    rsx! {
        link { rel: "stylesheet", href: CSS }
        ErrorToast { message: error_message }
        SidebarLayout {}
    }
}

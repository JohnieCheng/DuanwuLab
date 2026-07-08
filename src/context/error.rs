use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct GlobalErrorContext {
    pub message: Signal<Option<String>>,
}

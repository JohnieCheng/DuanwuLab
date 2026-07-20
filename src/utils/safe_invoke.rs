use dioxus::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::context::error::GlobalErrorContext;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

fn is_tauri() -> bool {
    js_sys::eval("typeof window.__TAURI__ !== 'undefined'")
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Call a Tauri backend command, serializing the request and deserializing
/// the response. Errors are automatically written into the global error context
/// so the toast in `App` picks them up.
///
/// Returns `None` on failure — the caller should treat that as "an error was
/// already reported".
pub async fn safe_invoke<T, REQ>(
    cmd: &str,
    args: REQ,
    mut error_ctx: GlobalErrorContext,
) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de>,
    REQ: serde::Serialize,
{
    // 0. Guard: skip if not running inside Tauri (e.g., dx serve)
    if !is_tauri() {
        error_ctx.message.set(Some(
            "Tauri runtime not available — run with `cargo tauri dev`, not `dx serve`".to_string(),
        ));
        return None;
    }

    // 1. Serialize request parameters to JS values
    let js_args = match serde_wasm_bindgen::to_value(&args) {
        Ok(v) => v,
        Err(e) => {
            error_ctx.message.set(Some(format!("Frontend Serialization Error: {}", e)));
            return None;
        }
    };

    // 2. Call the Tauri IPC bridge
    let raw = invoke(cmd, js_args).await;
    match raw {
        Ok(raw_js_val) => {
            // 3. Deserialize the successful response from the backend
            match serde_wasm_bindgen::from_value::<T>(raw_js_val) {
                Ok(data) => Some(data),
                Err(e) => {
                    error_ctx.message.set(Some(format!("Frontend Deserialization Error: {}", e)));
                    None
                }
            }
        }
        Err(js_err) => {
            // 4. Backend returned an error — route it into the global error context
            let err_msg = js_err.as_string().unwrap_or_else(|| "Unknown backend error".into());
            error_ctx.message.set(Some(err_msg));
            None
        }
    }
}

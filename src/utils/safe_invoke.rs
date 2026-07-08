use crate::context::error::GlobalErrorContext;
use dioxus::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn safe_invoke<T, REQ>(
    cmd: &str,
    args: REQ,
    mut error_ctx: GlobalErrorContext,
) -> Option<T>
where
    T: for<'de> serde::Deserialize<'de>,
    REQ: serde::Serialize,
{
    // 1. Serialize request parameters
    let js_args = match serde_wasm_bindgen::to_value(&args) {
        Ok(v) => v,
        Err(e) => {
            error_ctx.message.set(Some(format!("Frontend Serialization Error: {}", e)));
            return None;
        }
    };

    // 2. Call the underlying invoke (the invoke signature should be -> Result<JsValue, JsValue> here)
    match invoke(cmd, js_args).await {
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
            // 4. Global interception: Catch the Result::Err from the backend and insert it directly into the global context
            let err_msg = js_err.as_string().unwrap_or_else(|| "Unknown backend error".into());
            error_ctx.message.set(Some(err_msg));
            None
        }
    }
}

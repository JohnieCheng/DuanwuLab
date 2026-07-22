//! Common types shared between the Dioxus frontend and Tauri backend.
//!
//! Each tool's types live in their own module, gated by a cargo feature flag.
//! Add `shared-types = { path = "shared", features = ["<tool-name>"] }` to opt in.

#[cfg(feature = "image-compressor")]
pub mod image_compressor;

#[cfg(feature = "json-formatter")]
pub mod json_formatter;

use serde::{Deserialize, Serialize};

/// Shared preview data returned by Tauri backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct PreviewData {
    pub original: String,
    pub compressed: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub width: u32,
    pub height: u32,
}

/// Preview modal runtime state: data, asset URLs, slider position.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct PreviewState {
    pub data: PreviewData,
    pub original_url: String,
    pub compressed_url: String,
    pub slider_pos: f64,
}

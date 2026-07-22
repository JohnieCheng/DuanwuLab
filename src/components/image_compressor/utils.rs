/// Human-readable file size.
pub(super) fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Convert a local file path to a Tauri asset protocol URL.
pub(super) fn asset_url(path: &str) -> String {
    let escaped = path.replace('\'', "\\'");
    let js = format!("window.__TAURI__.core.convertFileSrc('{}')", escaped);
    js_sys::eval(&js).ok().and_then(|v| v.as_string()).unwrap_or_else(|| path.to_string())
}

/// Calculate space saving percentage.
pub(super) fn saved_percent(original: u64, compressed: u64) -> u32 {
    if original > compressed {
        ((original - compressed) as f64 / (original as f64).max(1.0) * 100.0) as u32
    } else {
        0
    }
}

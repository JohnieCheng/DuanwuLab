// ── Debounce / timer durations (milliseconds) ──

/// Debounce delay before triggering a search after the user stops typing.
pub(super) const SEARCH_DEBOUNCE_MS: u64 = 300;

/// Debounce delay before auto-enabling Smart Repair after input change.
pub(super) const AUTO_REPAIR_DEBOUNCE_MS: u64 = 500;

/// Debounce delay before the child component fires auto-format.
pub(super) const AUTO_FORMAT_DEBOUNCE_MS: u64 = 500;

/// How long the "✓ Copied" feedback stays visible.
pub(super) const COPY_FEEDBACK_MS: u64 = 1500;

/// Objects / arrays with no more than this many members are considered "small"
/// and auto-expanded one level when their parent is manually expanded.
pub(super) const AUTO_EXPAND_LIMIT: usize = 5;

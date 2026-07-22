use serde::{Deserialize, Serialize};

/// Result returned by the JSON formatting tool.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct JsonFormatResult {
    pub formatted: String,
    pub original_length: usize,
    pub formatted_length: usize,
    /// Whether Smart Repair was applied before formatting.
    pub repaired: bool,
}

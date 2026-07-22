use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Map key: `(file_path, quality)` serialized as `"quality::path"`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResultKey(pub String, pub u8);

impl Serialize for ResultKey {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{}::{}", self.1, self.0))
    }
}

impl<'de> Deserialize<'de> for ResultKey {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(d)?;
        if let Some((q, path)) = s.split_once("::") {
            Ok(ResultKey(path.to_string(), q.parse().unwrap_or(75)))
        } else {
            Ok(ResultKey(s, 75))
        }
    }
}

/// Image file selected by the user.
#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct PickedFile {
    pub path: String,
    pub size: u64,
    pub name: String,
}

/// Result of a single image compression operation.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CompressResult {
    pub original_path: String,
    pub compressed_path: String,
    pub name: String,
    pub original: u64,
    pub compressed: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub error: Option<String>,
    pub quality: u8,
}

/// Archived result stored on disk for history.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct CompressHistoryItem {
    pub results: Vec<CompressResult>,
    pub total_files: usize,
    pub total_original: u64,
    pub total_compressed: u64,
    pub created_at: String,
    pub state: ImageCompressorStateSnapshot,
}

/// Snapshot of compressor settings archived with history.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ImageCompressorStateSnapshot {
    pub quality: u8,
    pub compress_mode: String,
    pub output_dir: Option<String>,
    pub files: Vec<PickedFile>,
    pub results: Vec<CompressResult>,
}

// ── ImageCompressorState (shared by frontend & backend) ──────────────────

/// Lightweight, non-persisted state for the image compressor tool.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ImageCompressorState {
    #[serde(default)]
    pub files: Vec<PickedFile>,
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default = "default_mode")]
    pub compress_mode: String,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub results: HashMap<ResultKey, Result<CompressResult, String>>,
    /// Runtime-only: files currently being compressed.
    #[serde(skip, default)]
    #[allow(dead_code)]
    pub compressing_paths: HashSet<String>,
}

fn default_quality() -> u8 {
    75
}
fn default_mode() -> String {
    "fast".to_string()
}

impl Default for ImageCompressorState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            quality: default_quality(),
            compress_mode: default_mode(),
            output_dir: None,
            results: HashMap::new(),
            compressing_paths: HashSet::new(),
        }
    }
}

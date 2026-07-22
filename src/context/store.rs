use serde::{Deserialize, Serialize};
pub use shared_types::image_compressor::{
    CompressResult, ImageCompressorState, PickedFile, ResultKey,
};

/// Single source of truth for all tool state.
/// Persisted to disk via `save_store` / `load_store` Tauri commands.
#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct AppStore {
    pub compressor: ImageCompressorState,
}

/// Messages dispatched to the image-compression coroutine.
///
/// Heavy IPC tasks only; sync state mutations are handled directly in views.
pub enum CompressTaskMsg {
    AddImages,
    CompressAll,
}

/// System-level messages for the persistence coroutine.
pub enum SystemMsg {
    SaveToDisk,
}

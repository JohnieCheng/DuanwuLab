use crate::commands::{compress, json_format};

mod commands;
mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            json_format::format_json,
            compress::pick_image_files,
            compress::compress_single_image,
            compress::preview_compress,
            compress::pick_output_dir,
            store::load_store,
            store::save_store,
            store::add_history,
            store::open_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

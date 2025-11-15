use crate::core::dialog_handler::DialogHandler;
use std::fs;

#[tauri::command]
pub fn read_file_content(file_path: String) -> Result<String, String> {
    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file {}: {}", file_path, e)),
    }
}

#[tauri::command]
pub fn open_file_dialog() -> Result<String, String> {
    DialogHandler::select_video_file()
}
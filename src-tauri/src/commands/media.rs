use crate::core::image_handler::ImageHandler;
use crate::core::video_handler::VideoHandler;

#[tauri::command]
pub fn get_video_info(filename: &str) -> Result<String, String> {
    VideoHandler::get_video_info(filename)
}

#[tauri::command]
pub fn read_video_file(file_path: String) -> Result<String, String> {
    VideoHandler::read_video_file(&file_path)
}

#[tauri::command]
pub fn read_image_file(file_path: String) -> Result<String, String> {
    ImageHandler::read_image_info(&file_path)
}
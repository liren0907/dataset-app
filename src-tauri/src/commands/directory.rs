use crate::core::directory_handler::DirectoryHandler;
use crate::core::image_annotator::ImageAnnotator;

#[tauri::command]
pub fn get_directory_images(path: String) -> Result<String, String> {
    DirectoryHandler::get_directory_images(&path)
}

#[tauri::command]
pub fn get_paginated_images(path: String, page: usize, page_size: usize) -> Result<String, String> {
    DirectoryHandler::get_paginated_images(&path, page, page_size)
}

#[tauri::command]
pub fn get_image_details(path: String) -> Result<String, String> {
    DirectoryHandler::get_image_details(&path)
}

#[tauri::command]
pub fn auto_annotate_images(
    path: String,
    page: usize,
    page_size: usize,
) -> Result<String, String> {
    ImageAnnotator::auto_annotate_images(&path, page, page_size)
}
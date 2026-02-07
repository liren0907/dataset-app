use crate::core::dialog_handler::DialogHandler;
use std::fs;
use std::path::Path;

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

/// Check if a path exists (file or directory)
#[tauri::command]
pub fn path_exists(path: String) -> bool {
    Path::new(&path).exists()
}

/// Copy a directory tree from source to destination
#[tauri::command]
pub fn copy_directory(source: String, destination: String) -> Result<String, String> {
    let src_path = Path::new(&source);
    let dest_path = Path::new(&destination);

    if !src_path.exists() {
        return Err(format!("Source directory does not exist: {}", source));
    }

    // Create destination directory
    fs::create_dir_all(&dest_path)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    // Copy all files and subdirectories
    copy_dir_recursive(src_path, dest_path)
        .map_err(|e| format!("Failed to copy directory: {}", e))?;

    Ok(format!("Successfully copied to {}", destination))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }
    Ok(())
}

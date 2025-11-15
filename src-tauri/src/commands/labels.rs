use crate::core::labelme_viewer::LabelmeViewerModule;
use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn change_label_name(
    source_dir: String,
    output_dir: String,
    old_label: String,
    new_label: String,
) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create LabelMe viewer
    match LabelmeViewerModule::new(&source_dir) {
        Ok(viewer) => match viewer.change_label_name(&output_dir, &old_label, &new_label) {
            Ok(_) => Ok("Labels changed successfully".to_string()),
            Err(e) => Err(format!("Failed to change labels: {}", e)),
        },
        Err(e) => Err(format!("Failed to initialize LabelMe viewer: {}", e)),
    }
}

#[tauri::command]
pub fn clear_image_data(source_dir: String, output_dir: String) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create LabelMe viewer
    match LabelmeViewerModule::new(&source_dir) {
        Ok(viewer) => match viewer.clear_image_data(&output_dir) {
            Ok(_) => Ok("Image data cleared successfully".to_string()),
            Err(e) => Err(format!("Failed to clear image data: {}", e)),
        },
        Err(e) => Err(format!("Failed to initialize LabelMe viewer: {}", e)),
    }
}

#[tauri::command]
pub fn extract_labels(
    source_dir: String,
    output_dir: String,
    labels_str: String,
) -> Result<String, String> {
    // Parse labels from comma-separated string
    let labels: Vec<String> = labels_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if labels.is_empty() {
        return Err("No labels specified".to_string());
    }

    let base_output_path = PathBuf::from(&output_dir);
    let final_output_path;

    // Determine the final output path based on whether the base_output_path exists
    // Using metadata to check if it's a directory (or any file system object)
    // A simple `exists()` check is sufficient for this logic.
    if base_output_path.exists() {
        // If output_dir exists, create and use a 'labelme_output' subdirectory within it
        final_output_path = base_output_path.join("labelme_output");
    } else {
        // If output_dir does not exist, use it directly as the final path
        final_output_path = base_output_path;
    }

    // Create the final output directory (and any parent directories) if they don't exist
    if let Err(e) = std::fs::create_dir_all(&final_output_path) {
        return Err(format!(
            "Failed to create output directory {}: {}",
            final_output_path.display(),
            e
        ));
    }

    // Create LabelMe viewer
    match LabelmeViewerModule::new(&source_dir) {
        Ok(viewer) => {
            // Pass the final_output_path to the viewer's extract_labels method
            match viewer.extract_labels(
                final_output_path
                    .to_str()
                    .ok_or("Invalid final output path string")?,
                &labels,
            ) {
                Ok(_) => Ok("Labels extracted successfully".to_string()),
                Err(e) => Err(format!("Failed to extract labels: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to initialize LabelMe viewer: {}", e)),
    }
}
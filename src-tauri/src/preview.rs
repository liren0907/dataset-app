use serde_json::json;
use std::fs;
use std::path::Path;

/// Load annotation metadata for a single image
///
/// This function takes an image path, finds its corresponding JSON annotation file,
/// and returns the annotation metadata to the frontend for processing.
///
/// The function:
/// 1. Takes an image path as input
/// 2. Finds the corresponding JSON annotation file
/// 3. Loads and returns the annotation metadata
///
/// This provides a simple way for the frontend to access annotation data
/// without any complex processing or drawing operations.
///
/// # Arguments
/// * `image_path` - Path to the source image file
///
/// # Returns
/// JSON string containing the annotation metadata from the JSON file
#[tauri::command]
pub fn generate_single_annotated_preview(
    image_path: String,
) -> Result<String, String> {
    println!("Loading annotation metadata for: {}", image_path);

    // Find corresponding JSON file
    let image_path_obj = Path::new(&image_path);
    let file_stem = image_path_obj.file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid image path - cannot determine file stem")?;

    let json_path = image_path_obj.parent()
        .map(|p| p.join(format!("{}.json", file_stem)))
        .ok_or("Cannot determine JSON path")?;

    if !json_path.exists() {
        return Err(format!("JSON annotation file not found: {}", json_path.display()));
    }

    // Read and parse JSON file
    let json_content = fs::read_to_string(&json_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;

    let json_value: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON file: {}", e))?;

    // Return the annotation metadata
    let result = json!({
        "image_path": image_path,
        "json_path": json_path.to_string_lossy(),
        "annotation_metadata": json_value
    });

    println!("✅ Successfully loaded annotation metadata from: {}", json_path.display());
    Ok(result.to_string())
}

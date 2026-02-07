use crate::core::annotation_processor;
use crate::core::bounding_box_drawer;
use crate::core::image_annotator::ImageAnnotator;
use crate::core::polygon_drawer;
use crate::crop_remap;
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[tauri::command]
pub fn crop_and_remap_annotations(
    window: tauri::Window,
    source_dir: String,
    output_dir: String,
    parent_label: String,
    required_child_labels_str: String,
    padding_factor: f32,
) -> Result<(), String> {
    // Parse the comma-separated string into a vector of string references
    // Note: We need to own the strings for the thread move
    let required_child_labels: Vec<String> = required_child_labels_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if required_child_labels.is_empty() {
        return Err("No required child labels specified".to_string());
    }

    // Validate padding factor
    if padding_factor <= 0.0 || padding_factor > 5.0 {
        return Err("Padding factor must be between 0.1 and 5.0".to_string());
    }

    println!(
        "Received crop_and_remap request: source={}, output={}, parent_label={}, required_child={:?}, padding_factor={:.2}",
        source_dir, output_dir, parent_label, required_child_labels, padding_factor
    );

    // Spawn a background task
    tauri::async_runtime::spawn(async move {
        // Create a channel or shared state if needed, or simply let the processor emit events
        // Since the current processor is synchronous, we can run it here

        // We need to modify the processor to accept a callback or emit events
        // For now, let's wrap the blocking call and emit start/end,
        // but ideally we should pass a callback to the processor for granular updates.

        use tauri::Emitter;

        // Emit start event
        let _ = window.emit(
            "crop-progress",
            json!({
                "current": 0,
                "total": 0,
                "message": "Starting crop process..."
            }),
        );

        // Convert Vec<String> to Vec<&str> for the processor
        let child_labels_refs: Vec<&str> =
            required_child_labels.iter().map(|s| s.as_str()).collect();

        // Call the processor
        let window_clone = window.clone();
        let result = annotation_processor::process_parent_child_annotations(
            &source_dir,
            &output_dir,
            &parent_label,
            &child_labels_refs,
            padding_factor,
            Some(move |current: usize, total: usize, message: String| {
                let _ = window_clone.emit(
                    "crop-progress",
                    json!({
                        "current": current,
                        "total": total,
                        "message": message
                    }),
                );
            }),
        );

        match result {
            Ok(message) => {
                // Extract image count from message
                let image_count = if let Some(captures) = regex::Regex::new(r"(\d+)\s*image")
                    .ok()
                    .and_then(|re| re.captures(&message))
                {
                    captures
                        .get(1)
                        .map(|m: regex::Match| m.as_str().parse::<i32>().unwrap_or(0))
                        .unwrap_or(0)
                } else {
                    0
                };

                // Emit complete event
                let _ = window.emit(
                    "crop-complete",
                    json!({
                        "tempPath": output_dir, // Use camelCase to match frontend expectations
                        "imageCount": image_count,
                        "message": message
                    }),
                );
            }
            Err(e) => {
                // Emit error event
                let _ = window.emit(
                    "crop-error",
                    json!({
                        "message": e
                    }),
                );
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn generate_annotated_previews(
    source_dir: String,
    num_previews: usize,
    temp_dir: String,
) -> Result<String, String> {
    println!(
        "Generating {} annotated preview images from: {}",
        num_previews, source_dir
    );

    // Create temp directory if it doesn't exist with proper permissions
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return Err(format!("Failed to create temp directory: {}", e));
    }

    // Set proper permissions for the temp directory (readable by all)
    #[cfg(unix)]
    if let Ok(metadata) = fs::metadata(&temp_dir) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        if let Err(e) = fs::set_permissions(&temp_dir, permissions) {
            println!("Warning: Failed to set temp directory permissions: {}", e);
        }
    }

    // Get annotated images using the ImageAnnotator - process all annotation types
    let annotation_result = ImageAnnotator::auto_annotate_images(&source_dir, 1, 1000);
    let annotation_result = annotation_result?;
    let parsed_result: serde_json::Value = serde_json::from_str(&annotation_result)
        .map_err(|e| format!("Failed to parse annotation result: {}", e))?;

    let annotated_images_data = parsed_result["annotated_images"]
        .as_array()
        .ok_or("No annotated images found")?;

    // Filter images with annotations
    let mut annotated_images = Vec::new();
    for image in annotated_images_data {
        let path = image["path"].as_str().ok_or("Invalid image path")?;
        let has_json = image["has_json"].as_bool().unwrap_or(false);
        let empty_vec = vec![];
        let annotations = image["annotations"].as_array().unwrap_or(&empty_vec);

        if has_json && !annotations.is_empty() {
            annotated_images.push((path.to_string(), annotations.clone()));
        }
    }

    if annotated_images.is_empty() {
        return Err("No annotated images found".to_string());
    }

    // Shuffle and take up to num_previews
    let mut hasher = DefaultHasher::new();
    annotated_images.hash(&mut hasher);
    let seed = hasher.finish() as usize;

    // Simple shuffle using seed
    for i in (1..annotated_images.len()).rev() {
        let j = seed.wrapping_mul(i) % (i + 1);
        annotated_images.swap(i, j);
    }

    let selected_images = annotated_images.into_iter().take(num_previews);

    // Generate annotated preview images
    let mut preview_paths = Vec::new();
    for (image_path, _annotations) in selected_images {
        // Find corresponding JSON file
        let image_path_obj = Path::new(&image_path);
        let file_stem = image_path_obj
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid image path")?;

        let json_path = image_path_obj
            .parent()
            .map(|p| p.join(format!("{}.json", file_stem)))
            .ok_or("Cannot determine JSON path")?;

        if !json_path.exists() {
            continue; // Skip if no JSON file
        }

        // Generate preview filename
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let preview_filename = format!("preview_{}_{}.jpg", timestamp, preview_paths.len());
        let preview_path = Path::new(&temp_dir).join(preview_filename);

        // Read JSON to determine annotation types
        let json_content = match fs::read_to_string(&json_path) {
            Ok(content) => content,
            Err(e) => {
                println!(
                    "Warning: Failed to read JSON file {}: {}",
                    json_path.display(),
                    e
                );
                continue;
            }
        };

        let json_value: serde_json::Value = match serde_json::from_str(&json_content) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!(
                    "Warning: Failed to parse JSON file {}: {}",
                    json_path.display(),
                    e
                );
                continue;
            }
        };

        // Check if we have polygons or rectangles
        let shapes = match json_value.get("shapes").and_then(|s| s.as_array()) {
            Some(shapes_array) => shapes_array,
            None => {
                println!(
                    "Warning: No shapes found in JSON file {}",
                    json_path.display()
                );
                continue;
            }
        };

        let mut has_polygons = false;
        let mut has_rectangles = false;

        for shape in shapes {
            if let Some(shape_type) = shape.get("shape_type").and_then(|st| st.as_str()) {
                match shape_type {
                    "polygon" => has_polygons = true,
                    "rectangle" => has_rectangles = true,
                    _ => {}
                }
            }
        }

        // Draw annotations based on detected types - prioritize polygons if both exist
        let mut drawing_success = false;
        let json_filename = json_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid JSON filename")?;

        // Try polygon drawing first if polygons are present
        if has_polygons {
            if let Ok(_) =
                polygon_drawer::draw_polygons(&source_dir, &json_path.to_string_lossy(), &temp_dir)
            {
                let expected_output = Path::new(&temp_dir).join(format!(
                    "{}_polygons.jpg",
                    json_filename.replace(".json", "")
                ));
                if expected_output.exists() {
                    if let Err(e) = fs::rename(&expected_output, &preview_path) {
                        println!("Warning: Failed to rename polygon preview file: {}", e);
                    } else {
                        drawing_success = true;
                    }
                }
            }
        }

        // Try bounding box drawing if rectangles are present and polygon drawing didn't succeed
        if !drawing_success && has_rectangles {
            if let Ok(_) = bounding_box_drawer::draw_bounding_boxes(
                &source_dir,
                &json_path.to_string_lossy(),
                &temp_dir,
            ) {
                let expected_output = Path::new(&temp_dir)
                    .join(format!("{}_boxes.jpg", json_filename.replace(".json", "")));
                if expected_output.exists() {
                    if let Err(e) = fs::rename(&expected_output, &preview_path) {
                        println!("Warning: Failed to rename bounding box preview file: {}", e);
                    } else {
                        drawing_success = true;
                    }
                }
            }
        }

        if !drawing_success {
            println!("Warning: Failed to draw annotations for {}", image_path);
            continue;
        }

        // Add to preview paths if drawing was successful
        if drawing_success {
            // Set proper permissions for the generated preview file
            #[cfg(unix)]
            if let Ok(metadata) = fs::metadata(&preview_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o644); // rw-r--r--
                if let Err(e) = fs::set_permissions(&preview_path, permissions) {
                    println!("Warning: Failed to set preview file permissions: {}", e);
                }
            }
            preview_paths.push(preview_path.to_string_lossy().to_string());
        }
    }

    // Use annotation data directly (no preview paths needed)
    let mut annotated_images_result = Vec::new();
    let empty_annotations = Vec::new();

    for image in annotated_images_data {
        let has_json = image["has_json"].as_bool().unwrap_or(false);
        let annotations = image["annotations"]
            .as_array()
            .unwrap_or(&empty_annotations);

        // Only include images that have annotations
        if has_json && !annotations.is_empty() {
            annotated_images_result.push(image.clone());
        }
    }

    // Take only the first num_previews images
    let annotated_images_result = annotated_images_result
        .into_iter()
        .take(num_previews)
        .collect::<Vec<_>>();

    let result = json!({
        "annotated_images": annotated_images_result,
        "total": annotated_images_data.len(),
        "preview_count": annotated_images_result.len()
    });

    Ok(result.to_string())
}

#[tauri::command]
pub fn crop_remap_adapter(source_dir: String, num_previews: usize) -> Result<String, String> {
    crop_remap::crop_remap_adapter(source_dir, num_previews)
}

use crate::core::bounding_box_drawer;
use crate::core::image_annotator::ImageAnnotator;
use crate::core::polygon_drawer;
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

pub fn crop_remap_adapter(source_dir: String, num_previews: usize) -> Result<String, String> {
    println!(
        "Generating {} annotated preview images from: {} (CROP REMAP ADAPTER)",
        num_previews, source_dir
    );

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

    // Generate annotated preview images with detailed information
    let mut processed_images = Vec::new();

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

        // Generate unique identifier for this preview
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let preview_id = format!("preview_{}_{}", timestamp, processed_images.len());

        // Read and encode original image
        let original_image_data = match fs::read(&image_path) {
            Ok(data) => base64::engine::general_purpose::STANDARD.encode(&data),
            Err(e) => {
                println!(
                    "Warning: Failed to read original image {}: {}",
                    image_path, e
                );
                continue;
            }
        };

        // Read annotation metadata
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
        let mut annotation_type = "unknown";
        let json_filename = json_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid JSON filename")?;

        // Try polygon drawing first if polygons are present
        let mut annotated_image_data = String::new();
        if has_polygons {
            if let Ok(_) = polygon_drawer::draw_polygons(
                &source_dir,
                &json_path.to_string_lossy(),
                &source_dir, // Use source directory instead of temp
            ) {
                let expected_output = Path::new(&source_dir).join(format!(
                    "{}_polygons.jpg",
                    json_filename.replace(".json", "")
                ));
                if expected_output.exists() {
                    if let Ok(data) = fs::read(&expected_output) {
                        annotated_image_data = general_purpose::STANDARD.encode(&data);
                        drawing_success = true;
                        annotation_type = "polygon";
                        // Clean up the temporary file
                        let _ = fs::remove_file(&expected_output);
                    }
                }
            }
        }

        // Try bounding box drawing if rectangles are present and polygon drawing didn't succeed
        if !drawing_success && has_rectangles {
            if let Ok(_) = bounding_box_drawer::draw_bounding_boxes(
                &source_dir,
                &json_path.to_string_lossy(),
                &source_dir, // Use source directory instead of temp
            ) {
                let expected_output = Path::new(&source_dir)
                    .join(format!("{}_boxes.jpg", json_filename.replace(".json", "")));
                if expected_output.exists() {
                    if let Ok(data) = fs::read(&expected_output) {
                        annotated_image_data = general_purpose::STANDARD.encode(&data);
                        drawing_success = true;
                        annotation_type = "rectangle";
                        // Clean up the temporary file
                        let _ = fs::remove_file(&expected_output);
                    }
                }
            }
        }

        if !drawing_success {
            println!("Warning: Failed to draw annotations for {}", image_path);
            continue;
        }

        // Create processed image entry with all required data
        let processed_image = json!({
            "original_image_path": image_path,
            "original_image": format!("data:image/jpeg;base64,{}", original_image_data),
            "annotation_metadata": json_value,
            "modified_annotation_metadata": json_value, // For now, same as original (no modifications in adapter)
            "annotated_image": format!("data:image/jpeg;base64,{}", annotated_image_data),
            "annotation_type": annotation_type,
            "preview_id": preview_id
        });

        processed_images.push(processed_image);
    }

    // Return the new detailed format
    let result = json!({
        "processed_images": processed_images,
        "total_found": annotated_images_data.len(),
        "successfully_processed": processed_images.len(),
        "requested_count": num_previews
    });

    Ok(result.to_string())
}

use crate::labelme_types::{LabelMeFile, LabelMeShape, /* BoundingBox, */ get_bounding_box};
use std::fs;
use std::path::{Path, /* PathBuf */};
// use std::collections::HashMap;
// use image::GenericImageView; // Import GenericImageView trait
use glob::glob;

#[derive(Debug, Clone)]
struct ChildAnnotationInfo {
    original_shape: LabelMeShape,
    remapped_shape: LabelMeShape,
}

// Helper to sanitize filenames
fn sanitize_filename(name: &str) -> String {
    name.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect()
}

/// Processes annotations: crops image based on parent label, remaps child annotations.
///
/// # Arguments
/// * `source_dir` - Directory containing original images and LabelMe JSON files.
/// * `output_dir` - Directory where cropped images and new JSON files will be saved.
/// * `parent_label` - The label of the annotation to use for cropping (e.g., "person").
/// * `required_child_labels` - The labels of the child annotations to look for (OR logic - any of these).
/// * `padding_factor` - Factor to expand parent bounding box (1.0 = no padding, 1.2 = 20% larger).
///
/// # Returns
/// A `Result` with a success message summarizing the processing, or an error message.
pub fn process_parent_child_annotations(
    source_dir_str: &str,
    output_dir_str: &str,
    parent_label: &str,
    required_child_labels: &[&str],
    padding_factor: f32
) -> Result<String, String> {
    let source_dir = Path::new(source_dir_str);
    let output_dir = Path::new(output_dir_str);

    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(format!("Source directory not found or is not a directory: {}", source_dir_str));
    }
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory '{}': {}", output_dir_str, e))?;

    let pattern = format!("{}/**/*.json", source_dir_str.replace("\\", "/")); // Handle windows paths for glob
    let glob_results = glob(&pattern)
        .map_err(|e| format!("Failed to read glob pattern '{}': {}", pattern, e))?;

    let mut processed_files_count = 0;
    let mut total_processed_parents = 0;
    let mut error_files = Vec::new();

    for entry in glob_results {
        match entry {
            Ok(json_path) => {
                let file_name = json_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                println!("Processing JSON: {}", file_name);

                match process_single_file(&json_path, source_dir, output_dir, parent_label, required_child_labels, padding_factor) {
                    Ok(count) => {
                         if count > 0 {
                            processed_files_count += 1;
                            total_processed_parents += count;
                            println!(" -> Successfully processed {} parent instances in this file.", count);
                         } else {
                             println!(" -> No parent '{}' instances with required children {:?} found or processed in this file.", parent_label, required_child_labels);
                         }
                    }
                    Err(e) => {
                        eprintln!(" -> Error processing {}: {}", file_name, e);
                        error_files.push(format!("{}: {}", file_name, e));
                    }
                }
            }
            Err(e) => eprintln!("Error accessing path via glob: {:?}", e),
        }
    }

    let summary = format!(
        "Processing complete. Found valid parent annotations in {} files, successfully processing a total of {} parent instances.",
        processed_files_count, total_processed_parents
    );

    if error_files.is_empty() {
        Ok(summary)
    } else {
        Err(format!("{}\nEncountered errors in {} files:\n - {}", summary, error_files.len(), error_files.join("\n - ")))
    }
}

// Helper function needed by the updated process_single_file logic
fn required_child_label() -> &'static str {
    "safety_helmet"
}

fn process_single_file(
    json_path: &Path,
    source_dir: &Path,
    output_dir: &Path,
    parent_label: &str,
    required_child_labels: &[&str],
    padding_factor: f32
) -> Result<usize, String> { // Returns count of successfully processed *parent annotations*
    let json_content = fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    let original_labelme: LabelMeFile = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON content: {}", e))?;

    let mut processed_parent_count = 0;

    // --- Loop through all shapes to find potential parents ---
    for (parent_index, parent_shape) in original_labelme.shapes.iter().enumerate() {

        // --- 1. Check if it's the parent label we're looking for ---
        if parent_shape.label != parent_label {
            continue; // Skip if not the target parent label
        }

        println!(" -> Found potential parent '{}' at index {}", parent_label, parent_index);

        // --- 2. Calculate Parent Bounding Box ---
        let parent_bbox = match get_bounding_box(&parent_shape.points) {
            Some(bbox) => bbox,
            None => {
                eprintln!(" -> Skipping parent at index {}: shape has no points.", parent_index);
                continue; // Skip this parent if it has no points
            }
        };

        // --- 2.5. Expand Parent Bounding Box with Padding Factor ---
        let expanded_parent_bbox = if padding_factor != 1.0 {
            // Calculate center and current dimensions
            let center_x = (parent_bbox.x_min + parent_bbox.x_max) / 2.0;
            let center_y = (parent_bbox.y_min + parent_bbox.y_max) / 2.0;
            let current_width = parent_bbox.x_max - parent_bbox.x_min;
            let current_height = parent_bbox.y_max - parent_bbox.y_min;

            // Calculate new dimensions with padding
            let new_half_width = (current_width * padding_factor) / 2.0;
            let new_half_height = (current_height * padding_factor) / 2.0;

            // Create expanded bounding box (centered)
            crate::labelme_types::BoundingBox {
                x_min: center_x - new_half_width,
                y_min: center_y - new_half_height,
                x_max: center_x + new_half_width,
                y_max: center_y + new_half_height,
            }
        } else {
            parent_bbox // No expansion if padding_factor is 1.0
        };

        println!(" -> Parent bbox: ({:.1}, {:.1}) to ({:.1}, {:.1}), expanded to: ({:.1}, {:.1}) to ({:.1}, {:.1}) with padding factor {:.2}", 
                 parent_bbox.x_min, parent_bbox.y_min, parent_bbox.x_max, parent_bbox.y_max,
                 expanded_parent_bbox.x_min, expanded_parent_bbox.y_min, expanded_parent_bbox.x_max, expanded_parent_bbox.y_max,
                 padding_factor);

        // Clamp bbox to image dimensions to prevent cropping errors
        let parent_crop_x = expanded_parent_bbox.x_min.max(0.0).floor() as u32;
        let parent_crop_y = expanded_parent_bbox.y_min.max(0.0).floor() as u32;
        let parent_crop_width = (expanded_parent_bbox.x_max.min(original_labelme.image_width as f32).ceil() - parent_crop_x as f32).max(1.0) as u32;
        let parent_crop_height = (expanded_parent_bbox.y_max.min(original_labelme.image_height as f32).ceil() - parent_crop_y as f32).max(1.0) as u32;

        // Check for zero or invalid dimensions after clamping
        if parent_crop_width == 0 || parent_crop_height == 0 {
            eprintln!(" -> Skipping parent at index {}: Calculated crop dimensions are invalid (W: {}, H: {})", parent_index, parent_crop_width, parent_crop_height);
            continue; // Skip this parent if crop dimensions invalid
        }

        // --- 3. Identify Child Annotations & Check for Required Label (for *this* parent) ---
        let mut child_annotations_to_remap: Vec<&LabelMeShape> = Vec::new();
        let mut found_required_child = false;

        for (child_index, child_shape) in original_labelme.shapes.iter().enumerate() {
            if child_index == parent_index { continue; } // Skip the parent itself

            if let Some(child_bbox) = get_bounding_box(&child_shape.points) {
                // Check if the child's *bounding box* OVERLAPS with *this* parent's *bounding box*
                if child_bbox.overlaps(&expanded_parent_bbox) {
                    // 🔧 FIX: Only add child if it matches one of the required child labels
                    if required_child_labels.contains(&child_shape.label.as_str()) {
                        child_annotations_to_remap.push(child_shape); // Add only specified child labels
                        found_required_child = true;
                    }
                }
            }
        }

        // --- Check if the required child was found for *this* parent ---
        if !found_required_child {
            println!(
                " -> Parent '{}' at index {} found, but no required child '{}' inside its bbox. Skipping this instance.",
                parent_label, parent_index,
                required_child_labels.join(", ")
            );
            continue; // Skip processing this parent instance
        }

        // Check if after filtering, there are actually children to remap (should be true if found_required_child is true)
        if child_annotations_to_remap.is_empty() {
             println!(" -> Parent '{}' at index {} found required children '{}', but found no children to remap (this might indicate an issue). Skipping.", parent_label, parent_index, required_child_labels.join(", "));
             continue; // Skip this parent instance
        }

        println!(
            " -> Parent '{}' at index {} found with required children '{}'. Processing {} child annotations.",
            parent_label, parent_index,
            required_child_labels.join(", "),
            child_annotations_to_remap.len()
        );

        // --- 4. Load Original Image (only if needed) ---
        // Consider loading the image just once outside the loop if performance is critical
        // and if all crops come from the same original image path.
        let original_image_path_relative = Path::new(&original_labelme.image_path);
        let mut original_image_path = json_path.parent().unwrap_or(source_dir).join(original_image_path_relative);

        if !original_image_path.exists() {
             original_image_path = source_dir.join(original_image_path_relative);
             if !original_image_path.exists() {
                 // Log error for this parent instance, but continue loop for others
                 eprintln!(" -> Error for parent at index {}: Original image not found at expected paths for {}", parent_index, original_labelme.image_path);
                 continue;
            }
        }

        let img = match image::open(&original_image_path) {
             Ok(img_data) => img_data,
             Err(e) => {
                 eprintln!(" -> Error for parent at index {}: Failed to open original image '{}': {}", parent_index, original_image_path.display(), e);
                 continue; // Skip this parent instance
             }
        };

        // --- 5. Crop Image (for *this* parent) ---
        let cropped_img = img.crop_imm(
            parent_crop_x,
            parent_crop_y,
            parent_crop_width,
            parent_crop_height,
        );

        // --- 6. Save Cropped Image (unique name for *this* parent instance) ---
        let original_filename_stem = Path::new(&original_labelme.image_path)
                                            .file_stem()
                                            .unwrap_or_default()
                                            .to_string_lossy();
        let original_extension = Path::new(&original_labelme.image_path)
                                            .extension()
                                            .unwrap_or_default()
                                            .to_string_lossy();

        let cropped_filename_base = format!(
            "{}_crop_{}_{}", // Index makes it unique per parent instance
            original_filename_stem,
            sanitize_filename(parent_label),
            parent_index
        );
        let cropped_image_filename = format!("{}.{}", cropped_filename_base, original_extension);
        let cropped_image_path = output_dir.join(&cropped_image_filename);

        if let Err(e) = cropped_img.save(&cropped_image_path) {
             eprintln!(" -> Error for parent at index {}: Failed to save cropped image '{}': {}", parent_index, cropped_image_path.display(), e);
             continue; // Skip saving JSON if image save failed
        }
        println!(" -> Saved cropped image: {}", cropped_image_filename);

        // --- 7. Remap Child Coordinates (for children of *this* parent) ---
        let mut remapped_child_shapes: Vec<LabelMeShape> = Vec::new();
        for child_shape in child_annotations_to_remap {
            let mut remapped_shape = child_shape.clone();
            remapped_shape.points = child_shape.points.iter().map(|&[x, y]| {
                let remapped_x = (x - expanded_parent_bbox.x_min).max(0.0);
                let remapped_y = (y - expanded_parent_bbox.y_min).max(0.0);
                [remapped_x, remapped_y]
            }).collect();
            remapped_child_shapes.push(remapped_shape);
        }

        // --- 8. Create New LabelMe JSON (for *this* parent instance) ---
        let new_labelme_data = LabelMeFile {
            version: original_labelme.version.clone(),
            flags: original_labelme.flags.clone(),
            shapes: remapped_child_shapes,
            image_path: cropped_image_filename.clone(),
            image_data: None,
            image_height: cropped_img.height(),
            image_width: cropped_img.width(),
            extra: original_labelme.extra.clone(),
        };

        // --- 9. Save New JSON (for *this* parent instance) ---
        let new_json_filename = format!("{}.json", cropped_filename_base);
        let new_json_path = output_dir.join(&new_json_filename);
        let new_json_content = match serde_json::to_string_pretty(&new_labelme_data) {
             Ok(content) => content,
             Err(e) => {
                 eprintln!(" -> Error for parent at index {}: Failed to serialize new LabelMe data: {}", parent_index, e);
                 continue; // Skip saving this JSON
             }
        };

        if let Err(e) = fs::write(&new_json_path, new_json_content) {
             eprintln!(" -> Error for parent at index {}: Failed to write new JSON file '{}': {}", parent_index, new_json_path.display(), e);
             // Don't increment count if JSON save failed
        } else {
             println!(" -> Saved new JSON: {}", new_json_filename);
             processed_parent_count += 1; // Increment count for successfully processed parent
        }

    } // End loop through shapes

    Ok(processed_parent_count) // Return total count of parents processed successfully in this file
} 
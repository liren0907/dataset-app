// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod annotation_processor;
mod bounding_box_drawer;
mod data_visualizer;
mod dialog_handler;
mod directory_handler;
mod file_operations;
mod image_annotator;
mod image_handler;
mod labelme2yolo;
mod labelme_types;
mod labelme_viewer;
mod polygon_drawer;
mod video_handler;
mod yolo_to_labelme_converter;

use dialog_handler::DialogHandler;
use directory_handler::DirectoryHandler;
use glob::glob;
use image_annotator::ImageAnnotator;
use image_handler::ImageHandler;
use labelme2yolo::YoloExporter;
use labelme_viewer::LabelmeViewerModule;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use video_handler::VideoHandler;
use yolo_to_labelme_converter::YoloToLabelmeConverter;
use annotation_processor::process_parent_child_annotations;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            read_video_file,
            read_image_file,
            open_file_dialog,
            draw_bounding_boxes,
            draw_polygons,
            export_to_yolo,
            labelme2yolo::export_to_yolo_new,
            convert_to_labelme,
            change_label_name,
            clear_image_data,
            extract_labels,
            get_dataset_stats,
            visualize_dataset,
            get_directory_images,
            get_paginated_images,
            get_image_details,
            auto_annotate_images,
            get_labelme_summary,
            crop_and_remap_annotations,
            generate_annotated_previews
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// // Mock main for testing
// fn main() {
//     println!("--- Testing crop_and_remap_annotations ---");

//     // --- Example Input Arguments ---
//     // TODO: Create this directory and place sample images + annotation files inside.
//     // Ensure some JSON files contain a "person" label with other labels nested inside.
//     let source_dir = String::from("/Users/admin/Desktop/wsp_t3_data/A2_T002_annotated_labelme_v3");

//     // TODO: Create this base directory.
//     let output_dir = String::from("/Users/admin/Desktop/wsp_t3_data/output");

//     // TODO: Set the label you want to crop around.
//     let parent_label = String::from("person");
    
//     // Define multiple required child labels (OR logic - any of these safety items)
//     let required_child_labels = vec!["safety_helmet", "reflective_vest", "body_harness"];
//     // --- End of Example Input Arguments ---

//     // Ensure directories exist for the test
//     if let Err(e) = std::fs::create_dir_all(&source_dir) {
//         eprintln!("Failed to create test source directory '{}': {}. Please create it manually.", source_dir, e);
//         return;
//     }
//      if let Err(e) = std::fs::create_dir_all(&output_dir) {
//         eprintln!("Failed to create test output directory '{}': {}. Please create it manually.", output_dir, e);
//         return;
//     }
//     println!("Test directories ensured/created.");
//     println!("Please add test images and JSON files to: {}", source_dir);
//     println!("Press Enter to continue after adding test files...");
//     let mut line = String::new();
//     std::io::stdin().read_line(&mut line).expect("Failed to read line");

//     println!("\n--- Source Directory Statistics ---");
//     match get_labelme_summary(&source_dir) {
//         Ok(summary_json) => {
//             match serde_json::from_str::<serde_json::Value>(&summary_json) {
//                 Ok(summary) => {
//                     println!("Total Images (JSON files): {}", summary["total_images"]);
//                     println!("Images with Annotations: {}", summary["images_with_annotations"]);
//                     println!("Total Annotations: {}", summary["total_annotations"]);
//                     println!("Unique Labels: {}", summary["unique_labels"]);
//                     if let Some(label_counts) = summary["label_counts"].as_object() {
//                         if !label_counts.is_empty() {
//                             println!("Label Counts:");
//                             for (label, count) in label_counts {
//                                 println!("  - {}: {}", label, count);
//                             }
//                         } else {
//                             println!("Label Counts: (No labels found)");
//                         }
//                     } else {
//                          println!("Could not read label counts from summary.");
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("Error parsing summary JSON: {}", e);
//                 }
//             }
//         }
//         Err(e) => {
//             eprintln!("Error getting dataset summary: {}", e);
//             // Optionally decide whether to continue or return if summary fails
//         }
//     }
//     println!("-----------------------------------\n");

//     println!("Source Directory: {}", source_dir);
//     println!("Output Directory: {}", output_dir);
//     println!("Parent Label: {}", parent_label);
//     println!("Required Child Labels: {:?}", required_child_labels);
//     println!("Calling process_parent_child_annotations...");

//     // Call the function directly for testing - required_child_labels is already Vec<&str>
//     let result = process_parent_child_annotations(
//         &source_dir,
//         &output_dir,
//         &parent_label,
//         &required_child_labels,
//         1.2  // padding_factor: 20% larger bounding box
//     );

//     // Handle the result
//     match result {
//         Ok(success_message) => {
//             println!("\n--- Success ---");
//             println!("{}", success_message);
//         }
//         Err(error_message) => {
//             eprintln!("\n--- Error ---");
//             eprintln!("{}", error_message);
//             // std::process::exit(1); // Optional: exit with error code
//         }
//     }

//     println!("\n--- Test Finished ---");
// }

#[tauri::command]
fn get_video_info(filename: &str) -> Result<String, String> {
    VideoHandler::get_video_info(filename)
}

#[tauri::command]
fn read_video_file(file_path: String) -> Result<String, String> {
    VideoHandler::read_video_file(&file_path)
}

#[tauri::command]
fn read_image_file(file_path: String) -> Result<String, String> {
    ImageHandler::read_image_info(&file_path)
}

#[tauri::command]
fn open_file_dialog() -> Result<String, String> {
    DialogHandler::select_video_file()
}

#[tauri::command]
fn draw_bounding_boxes(
    source_dir: String,
    json_path: String,
    output_dir: String,
) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Draw bounding boxes
    match bounding_box_drawer::draw_bounding_boxes(&source_dir, &json_path, &output_dir) {
        Ok(_) => Ok("Bounding boxes drawn successfully".to_string()),
        Err(e) => Err(format!("Failed to draw bounding boxes: {}", e)),
    }
}

#[tauri::command]
fn draw_polygons(
    source_dir: String,
    json_path: String,
    output_dir: String,
) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    match polygon_drawer::draw_polygons(&source_dir, &json_path, &output_dir) {
        Ok(_) => Ok("Polygons drawn successfully".to_string()),
        Err(e) => Err(format!("Failed to draw polygons: {}", e)),
    }
}

#[tauri::command]
fn visualize_dataset(
    source_dir: String,
    output_dir: String,
    annotation_type: String,
    save_output: bool,
) -> Result<String, String> {
    // Validate parameters
    if !Path::new(&source_dir).exists() {
        return Err(format!("Source directory does not exist: {}", source_dir));
    }

    if annotation_type != "bounding_box" && annotation_type != "polygon" {
        return Err(format!(
            "Invalid annotation type: {}. Must be 'bounding_box' or 'polygon'",
            annotation_type
        ));
    }

    // Only create output directory if we're saving output
    if save_output {
        if let Err(e) = fs::create_dir_all(&output_dir) {
            return Err(format!("Failed to create output directory: {}", e));
        }
    } else {
        // If not saving output, we'll just return information about annotations
        println!("Visualization will be processed but not saved (save_output=false)");
    }

    // Count of successfully processed files
    let mut success_count = 0;
    let mut error_count = 0;
    let mut annotation_count = 0;

    // Find all JSON files in the source directory using glob pattern
    let pattern = format!("{}/**/*.json", source_dir);
    match glob(&pattern) {
        Ok(paths) => {
            for path_result in paths {
                if let Ok(path) = path_result {
                    let json_path = path.to_str().unwrap_or_default();

                    if !save_output {
                        // Just count the JSON files without processing them
                        success_count += 1;

                        // Optionally, we could parse the JSON and count shapes here
                        if let Ok(json_content) = fs::read_to_string(&path) {
                            if let Ok(json) =
                                serde_json::from_str::<serde_json::Value>(&json_content)
                            {
                                if let Some(shapes) = json.get("shapes").and_then(|s| s.as_array())
                                {
                                    annotation_count += shapes.len();
                                }
                            }
                        }
                        continue;
                    }

                    // Process based on annotation type
                    let result = if annotation_type == "bounding_box" {
                        bounding_box_drawer::draw_bounding_boxes(
                            &source_dir,
                            json_path,
                            &output_dir,
                        )
                    } else {
                        polygon_drawer::draw_polygons(&source_dir, json_path, &output_dir)
                    };

                    match result {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            error_count += 1;
                            println!("Error processing {}: {}", json_path, e);
                        }
                    }
                }
            }

            if !save_output {
                // When not saving, report the files found and annotation count
                Ok(format!(
                    "Found {} JSON files with {} annotations (save_output=false)",
                    success_count, annotation_count
                ))
            } else if success_count > 0 {
                Ok(format!(
                    "Successfully visualized {} files (with {} errors)",
                    success_count, error_count
                ))
            } else if error_count > 0 {
                Err(format!(
                    "Failed to visualize any files (encountered {} errors)",
                    error_count
                ))
            } else {
                Ok("No JSON files found to visualize".to_string())
            }
        }
        Err(e) => Err(format!("Error finding JSON files: {}", e)),
    }
}

#[tauri::command]
fn export_to_yolo(
    source_dir: String,
    output_dir: String,
    train_ratio: f32,
    val_ratio: f32,
    test_ratio: f32,
    shape: String,
    specific_labels: bool,
    class_names_str: String,
) -> Result<String, String> {
    // Parse class names from comma-separated string
    let class_names = if specific_labels && !class_names_str.is_empty() {
        class_names_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create exporter
    let exporter = YoloExporter::new(
        &source_dir,
        &output_dir,
        train_ratio,
        val_ratio,
        test_ratio,
        &shape,
        specific_labels,
        class_names,
    );

    // Export
    match exporter.export() {
        Ok(_) => Ok("Dataset exported to YOLO format successfully".to_string()),
        Err(e) => Err(format!("Failed to export to YOLO format: {}", e)),
    }
}

#[tauri::command]
fn convert_to_labelme(
    source_dir: String,
    output_dir: String,
    version: String,
    class_label_file: String,
) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create converter
    let mut converter =
        YoloToLabelmeConverter::new(&source_dir, &output_dir, &version, &class_label_file);

    // Convert
    match converter.convert() {
        Ok(_) => Ok("Dataset converted to LabelMe format successfully".to_string()),
        Err(e) => Err(format!("Failed to convert to LabelMe format: {}", e)),
    }
}

#[tauri::command]
fn change_label_name(
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
fn clear_image_data(source_dir: String, output_dir: String) -> Result<String, String> {
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
fn extract_labels(
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

#[tauri::command]
fn get_dataset_stats(source_dir: String) -> Result<String, String> {
    // Create LabelMe viewer
    match LabelmeViewerModule::new(&source_dir) {
        Ok(mut viewer) => {
            let stats = viewer.get_statistics();
            match serde_json::to_string(&stats) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize statistics: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to initialize LabelMe viewer: {}", e)),
    }
}

#[tauri::command]
fn get_directory_images(path: String) -> Result<String, String> {
    DirectoryHandler::get_directory_images(&path)
}

#[tauri::command]
fn get_paginated_images(path: String, page: usize, pageSize: usize) -> Result<String, String> {
    DirectoryHandler::get_paginated_images(&path, page, pageSize)
}

#[tauri::command]
fn get_image_details(path: String) -> Result<String, String> {
    DirectoryHandler::get_image_details(&path)
}

#[tauri::command]
fn auto_annotate_images(
    path: String,
    page: usize,
    pageSize: usize,
    annotationType: String,
) -> Result<String, String> {
    ImageAnnotator::auto_annotate_images(&path, page, pageSize, &annotationType)
}

#[tauri::command]
fn get_labelme_summary(path: &str) -> Result<String, String> {
    println!("Generating LabelMe summary for: {}", path);

    // Validate input path
    if !std::path::Path::new(path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Create a summary of the dataset
    let mut total_images = 0;
    let mut images_with_annotations = 0;
    let mut total_annotations = 0;
    let mut label_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut annotation_types: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Pattern to find JSON files (LabelMe annotations)
    let pattern = format!("{}/**/*.json", path);
    match glob::glob(&pattern) {
        Ok(paths) => {
            for entry in paths {
                match entry {
                    Ok(json_path) => {
                        // Count total JSON files
                        total_images += 1;

                        // Read and parse the JSON file
                        if let Ok(content) = std::fs::read_to_string(&json_path) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                let _has_annotations = if let Some(shapes) =
                                    json["shapes"].as_array()
                                {
                                    if !shapes.is_empty() {
                                        images_with_annotations += 1;

                                        // Process each annotation
                                        for shape in shapes {
                                            total_annotations += 1;

                                            // Count labels
                                            if let Some(label) = shape["label"].as_str() {
                                                *label_counts
                                                    .entry(label.to_string())
                                                    .or_insert(0) += 1;
                                            }

                                            // Track annotation types
                                            if let Some(shape_type) = shape["shape_type"].as_str() {
                                                annotation_types.insert(shape_type.to_string());
                                            }
                                        }

                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                };
                                // Use _has_annotations to silence the warning if it's not used later.
                                // If you intended to use it, uncomment the line below and integrate it.
                                // let _ = has_annotations;
                            }
                        }
                    }
                    Err(e) => println!("Error accessing path: {:?}", e),
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read directory: {}", e));
        }
    }

    // Create summary object
    let summary = serde_json::json!({
        "total_images": total_images,
        "images_with_annotations": images_with_annotations,
        "total_annotations": total_annotations,
        "unique_labels": label_counts.len(),
        "label_counts": label_counts,
        "annotation_types": annotation_types.into_iter().collect::<Vec<_>>(),
    });

    Ok(summary.to_string())
}

// Added Tauri command wrapper
#[tauri::command]
fn crop_and_remap_annotations(
    source_dir: String,
    output_dir: String,
    parent_label: String,
    required_child_labels_str: String,
    padding_factor: f32,
) -> Result<String, String> {
    // Parse the comma-separated string into a vector of string references
    let required_child_labels: Vec<&str> = required_child_labels_str
        .split(',')
        .map(|s| s.trim())
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
    
    // Call the actual processing logic, passing the new argument
    annotation_processor::process_parent_child_annotations(
        &source_dir,
        &output_dir,
        &parent_label,
        &required_child_labels,
        padding_factor,
    )
}

#[tauri::command]
fn generate_annotated_previews(
    source_dir: String,
    num_previews: usize,
    temp_dir: String,
) -> Result<String, String> {
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;

    println!("Generating {} annotated preview images from: {}", num_previews, source_dir);

    // Create temp directory if it doesn't exist with proper permissions
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        return Err(format!("Failed to create temp directory: {}", e));
    }
    
    // Set proper permissions for the temp directory (readable by all)
    if let Ok(metadata) = fs::metadata(&temp_dir) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        if let Err(e) = fs::set_permissions(&temp_dir, permissions) {
            println!("Warning: Failed to set temp directory permissions: {}", e);
        }
    }

    // Get annotated images using the ImageAnnotator - try both polygon and bounding_box types
    let mut annotation_result = ImageAnnotator::auto_annotate_images(&source_dir, 1, 1000, "polygon");
    if annotation_result.is_err() || annotation_result.as_ref().unwrap().contains("No annotated images found") {
        annotation_result = ImageAnnotator::auto_annotate_images(&source_dir, 1, 1000, "bounding_box");
    }
    let annotation_result = annotation_result?;
    let parsed_result: serde_json::Value = serde_json::from_str(&annotation_result)
        .map_err(|e| format!("Failed to parse annotation result: {}", e))?;

    let annotated_images_data = parsed_result["annotated_images"].as_array()
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
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    annotated_images.hash(&mut hasher);
    let seed = hasher.finish() as usize;

    // Simple shuffle using seed
    for i in (1..annotated_images.len()).rev() {
        let j = (seed.wrapping_mul(i) % (i + 1));
        annotated_images.swap(i, j);
    }

    let selected_images = annotated_images.into_iter().take(num_previews);

    // Generate annotated preview images
    let mut preview_paths = Vec::new();
    for (image_path, _annotations) in selected_images {
        // Find corresponding JSON file
        let image_path_obj = Path::new(&image_path);
        let file_stem = image_path_obj.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid image path")?;

        let json_path = image_path_obj.parent()
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
                println!("Warning: Failed to read JSON file {}: {}", json_path.display(), e);
                continue;
            }
        };

        let json_value: serde_json::Value = match serde_json::from_str(&json_content) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Warning: Failed to parse JSON file {}: {}", json_path.display(), e);
                continue;
            }
        };

        // Check if we have polygons or rectangles
        let shapes = match json_value.get("shapes").and_then(|s| s.as_array()) {
            Some(shapes_array) => shapes_array,
            None => {
                println!("Warning: No shapes found in JSON file {}", json_path.display());
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
        let json_filename = json_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid JSON filename")?;

        // Try polygon drawing first if polygons are present
        if has_polygons {
            if let Ok(_) = polygon_drawer::draw_polygons(
                &source_dir,
                &json_path.to_string_lossy(),
                &temp_dir,
            ) {
                let expected_output = Path::new(&temp_dir).join(format!("{}_polygons.jpg", json_filename.replace(".json", "")));
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
                let expected_output = Path::new(&temp_dir).join(format!("{}_boxes.jpg", json_filename.replace(".json", "")));
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

    let result = json!({
        "success": true,
        "preview_count": preview_paths.len(),
        "previews": preview_paths
    });

    Ok(result.to_string())
}

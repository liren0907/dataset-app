use crate::core::bounding_box_drawer;
use crate::core::polygon_drawer;
use glob::glob;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn visualize_dataset(
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

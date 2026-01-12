use crate::core::labelme_viewer::LabelmeViewerModule;
use glob::glob;
use std::collections::{HashMap, HashSet};

#[tauri::command]
pub fn get_dataset_stats(source_dir: String) -> Result<String, String> {
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
pub fn get_labelme_summary(path: &str) -> Result<String, String> {
    println!("Generating LabelMe summary for: {}", path);

    // Validate input path
    if !std::path::Path::new(path).exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Create a summary of the dataset
    let mut total_images = 0;
    let mut images_with_annotations = 0;
    let mut total_annotations = 0;
    let mut label_counts: HashMap<String, usize> = HashMap::new();
    let mut annotation_types: HashSet<String> = HashSet::new();

    // Pattern to find JSON files (LabelMe annotations)
    let pattern = format!("{}/**/*.json", path);
    match glob(&pattern) {
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

#[derive(Debug, serde::Deserialize)]
pub struct AnnotationData {
    pub label: String,
    pub shape_type: String,
    pub points: Vec<Vec<f32>>,
}

#[tauri::command]
pub fn save_annotation(
    image_path: String,
    annotations: Vec<AnnotationData>,
) -> Result<String, String> {
    println!("Saving annotations for: {}", image_path);

    let path = std::path::Path::new(&image_path);
    let parent = path.parent().ok_or("Invalid image path")?;
    let stem = path.file_stem().ok_or("Invalid filename")?;
    let json_path = parent.join(format!("{}.json", stem.to_string_lossy()));

    // 1. Read existing JSON or create basic structure
    let mut json_data: serde_json::Value = if json_path.exists() {
        let content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read existing JSON: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse existing JSON: {}", e))?
    } else {
        serde_json::json!({
            "version": "5.0.1",
            "flags": {},
            "shapes": [],
            "imagePath": path.file_name().unwrap().to_string_lossy(),
            "imageData": null,
            "imageHeight": 0, // Should ideally be updated if known
            "imageWidth": 0
        })
    };

    // 2. Convert annotations to LabelMe shapes
    let new_shapes: Vec<serde_json::Value> = annotations
        .into_iter()
        .map(|ann| {
            serde_json::json!({
                "label": ann.label,
                "points": ann.points,
                "group_id": null,
                "description": "",
                "shape_type": ann.shape_type,
                "flags": {}
            })
        })
        .collect();

    // 3. Update shapes in JSON
    if let Some(obj) = json_data.as_object_mut() {
        obj.insert("shapes".to_string(), serde_json::Value::Array(new_shapes));
    }

    // 4. Write back to file
    let new_content = serde_json::to_string_pretty(&json_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    std::fs::write(&json_path, new_content)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;

    Ok("Annotations saved successfully".to_string())
}

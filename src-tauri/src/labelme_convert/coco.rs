// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::conversion::{
    calculate_coco_bbox, calculate_polygon_area, determine_split, flatten_polygon, hash_string,
    rectangle_to_polygon, Split,
};
use crate::labelme_convert::io::{
    copy_image, extract_embedded_image, find_json_files, read_labelme_json, resolve_image_path,
    setup_coco_directories, write_file,
};
use crate::labelme_convert::types::{
    CocoOutputDirs, ConversionResult, InputAnnotationFormat, InvalidAnnotation, InvalidReason,
    ProcessingStats, Shape,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ============================================================================
// COCO Data Structures
// ============================================================================

/// COCO dataset info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoInfo {
    pub year: u32,
    pub version: String,
    pub description: String,
    pub contributor: String,
    pub url: String,
    pub date_created: String,
}

impl Default for CocoInfo {
    fn default() -> Self {
        Self {
            year: 2024,
            version: "1.0".to_string(),
            description: "Exported from LabelMe".to_string(),
            contributor: "dataset-app".to_string(),
            url: String::new(),
            date_created: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        }
    }
}

/// COCO license info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoLicense {
    pub id: u32,
    pub name: String,
    pub url: String,
}

impl Default for CocoLicense {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Unknown".to_string(),
            url: String::new(),
        }
    }
}

/// COCO category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoCategory {
    pub id: u32,
    pub name: String,
    pub supercategory: String,
}

/// COCO image info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoImage {
    pub id: u32,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub license: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flickr_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coco_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_captured: Option<String>,
}

/// COCO annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoAnnotation {
    pub id: u32,
    pub image_id: u32,
    pub category_id: u32,
    pub bbox: [f64; 4],
    pub area: f64,
    pub iscrowd: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation: Option<Vec<Vec<f64>>>,
}

/// Complete COCO dataset file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CocoDataset {
    pub info: CocoInfo,
    pub licenses: Vec<CocoLicense>,
    pub categories: Vec<CocoCategory>,
    pub images: Vec<CocoImage>,
    pub annotations: Vec<CocoAnnotation>,
}

impl Default for CocoDataset {
    fn default() -> Self {
        Self {
            info: CocoInfo::default(),
            licenses: vec![CocoLicense::default()],
            categories: Vec::new(),
            images: Vec::new(),
            annotations: Vec::new(),
        }
    }
}

// ============================================================================
// COCO Conversion
// ============================================================================

/// Main COCO dataset conversion function
pub fn convert_to_coco(config: &ConversionConfig) -> ConversionResult {
    // Validate configuration
    if let Err(e) = config.validate() {
        return ConversionResult::failure(vec![e]);
    }

    // Set up output directories
    let output_dirs = match setup_coco_directories(config) {
        Ok(dirs) => dirs,
        Err(e) => {
            return ConversionResult::failure(vec![format!(
                "Failed to create output directories: {}",
                e
            )])
        }
    };

    let mut stats = ProcessingStats::new();
    let mut errors = Vec::new();
    let mut label_map: HashMap<String, usize> = HashMap::new();
    let mut processed_images: HashSet<String> = HashSet::new();

    // Initialize label map from config if provided
    for (id, label) in config.label_list.iter().enumerate() {
        label_map.insert(label.clone(), id);
    }

    // Find all JSON files
    let json_files = find_json_files(&config.input_dir);
    stats.total_files = json_files.len();

    // First pass: gather all labels if deterministic
    if config.deterministic_labels && config.label_list.is_empty() {
        gather_labels_coco(&json_files, &mut label_map);
    }

    // Split datasets
    let mut train_dataset = CocoDataset::default();
    let mut val_dataset = CocoDataset::default();
    let mut test_dataset = CocoDataset::default();

    let mut image_id_counter = config.start_image_id;
    let mut annotation_id_counter = config.start_annotation_id;
    let mut skipped_labels: HashSet<String> = HashSet::new();

    // Get the pre-detected input format from config
    let input_format = config.detected_input_format.unwrap_or(InputAnnotationFormat::Unknown);

    // Process each JSON file
    for json_path in &json_files {
        match process_single_file_coco(
            json_path,
            config,
            &output_dirs,
            &mut label_map,
            &mut processed_images,
            &mut skipped_labels,
            &mut train_dataset,
            &mut val_dataset,
            &mut test_dataset,
            &mut image_id_counter,
            &mut annotation_id_counter,
            input_format,
        ) {
            Ok((annotation_count, skipped_count, invalid_list)) => {
                stats.increment_processed();
                stats.add_annotations(annotation_count);
                stats.add_skipped_annotations(skipped_count);
                for invalid in invalid_list {
                    stats.add_invalid_annotation(invalid);
                }
            }
            Err(e) => {
                stats.increment_failed();
                errors.push(format!("{}: {}", json_path.display(), e));
            }
        }
    }

    // Build categories
    let categories = build_categories(&label_map);
    train_dataset.categories = categories.clone();
    val_dataset.categories = categories.clone();
    test_dataset.categories = categories;

    // Update stats with labels
    for label in label_map.keys() {
        stats.add_label(label.clone());
    }

    // Add skipped labels to stats
    for label in skipped_labels {
        stats.add_skipped_label(label);
    }

    // Write COCO JSON files
    if let Err(e) = write_coco_json(&output_dirs.annotations_dir.join("instances_train.json"), &train_dataset) {
        errors.push(format!("Failed to write train annotations: {}", e));
    }

    if let Err(e) = write_coco_json(&output_dirs.annotations_dir.join("instances_val.json"), &val_dataset) {
        errors.push(format!("Failed to write val annotations: {}", e));
    }

    if config.has_test_split() {
        if let Err(e) = write_coco_json(&output_dirs.annotations_dir.join("instances_test.json"), &test_dataset) {
            errors.push(format!("Failed to write test annotations: {}", e));
        }
    }

    if errors.is_empty() {
        ConversionResult::success(output_dirs.base_dir.to_string_lossy().to_string(), stats)
    } else {
        let mut result =
            ConversionResult::success(output_dirs.base_dir.to_string_lossy().to_string(), stats);
        result.errors = errors;
        result
    }
}

/// Gather all unique labels from JSON files
fn gather_labels_coco(json_files: &[std::path::PathBuf], label_map: &mut HashMap<String, usize>) {
    let mut all_labels: HashSet<String> = HashSet::new();

    for json_path in json_files {
        if let Ok(annotation) = read_labelme_json(json_path) {
            for shape in &annotation.shapes {
                all_labels.insert(shape.label.clone());
            }
        }
    }

    let mut sorted_labels: Vec<_> = all_labels.into_iter().collect();
    sorted_labels.sort();

    for (id, label) in sorted_labels.into_iter().enumerate() {
        label_map.insert(label, id);
    }
}

/// Process a single LabelMe JSON file for COCO
/// Returns (annotation_count, skipped_count, invalid_annotations)
#[allow(clippy::too_many_arguments)]
fn process_single_file_coco(
    json_path: &Path,
    config: &ConversionConfig,
    output_dirs: &CocoOutputDirs,
    label_map: &mut HashMap<String, usize>,
    processed_images: &mut HashSet<String>,
    skipped_labels: &mut HashSet<String>,
    train_dataset: &mut CocoDataset,
    val_dataset: &mut CocoDataset,
    test_dataset: &mut CocoDataset,
    image_id_counter: &mut u32,
    annotation_id_counter: &mut u32,
    input_format: InputAnnotationFormat,
) -> Result<(usize, usize, Vec<InvalidAnnotation>), String> {
    // Read and parse JSON
    let annotation = read_labelme_json(json_path)?;

    // Resolve image path
    let image_path = resolve_image_path(json_path, &annotation.image_path);

    // Track processed images
    let image_key = image_path.to_string_lossy().to_string();
    processed_images.insert(image_key.clone());

    // Determine split
    let path_hash = hash_string(&image_key);
    let split = determine_split(path_hash, config.val_size, config.test_size);

    // Get output directory for this split
    let images_dir = get_split_images_dir(output_dirs, split);

    // Add labels to map if needed
    if config.label_list.is_empty() && !config.deterministic_labels {
        for shape in &annotation.shapes {
            if !label_map.contains_key(&shape.label) {
                let next_id = label_map.len();
                label_map.insert(shape.label.clone(), next_id);
            }
        }
    }

    // Get image filename
    let file_name = image_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.jpg".to_string());

    // Get JSON filename for error reporting
    let json_file_name = json_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.json".to_string());

    // Copy or extract image
    if let Some(image_data) = &annotation.image_data {
        let dest_path = images_dir.join(&file_name);
        extract_embedded_image(image_data, &dest_path)?;
    } else if image_path.exists() {
        copy_image(&image_path, images_dir)
            .map_err(|e| format!("Failed to copy image: {}", e))?;
    } else {
        return Err(format!("Image file not found: {}", image_path.display()));
    }

    // Create COCO image entry
    let image_id = *image_id_counter;
    *image_id_counter += 1;

    let coco_image = CocoImage {
        id: image_id,
        file_name: file_name.clone(),
        width: annotation.image_width,
        height: annotation.image_height,
        license: 1,
        flickr_url: None,
        coco_url: None,
        date_captured: None,
    };

    // Create COCO annotations
    let mut coco_annotations = Vec::new();
    let mut skipped_count = 0;
    let mut invalid_annotations = Vec::new();

    for shape in &annotation.shapes {
        if let Some(class_id) = label_map.get(&shape.label) {
            // Validate points count based on detected input format
            if let Err(reason) = validate_shape_points(shape, input_format) {
                invalid_annotations.push(InvalidAnnotation {
                    file: json_file_name.clone(),
                    label: shape.label.clone(),
                    reason: reason.as_str(),
                    shape_type: shape.shape_type.clone(),
                    points_count: shape.points.len(),
                });
                skipped_count += 1;
                continue;
            }

            if let Some(coco_ann) = shape_to_coco_annotation(
                shape,
                image_id,
                (*class_id + 1) as u32, // COCO category IDs are 1-indexed
                *annotation_id_counter,
                config,
            ) {
                coco_annotations.push(coco_ann);
                *annotation_id_counter += 1;
            }
        } else {
            // Label not in the predefined list
            skipped_labels.insert(shape.label.clone());
            skipped_count += 1;
        }
    }

    let annotation_count = coco_annotations.len();

    // Add to appropriate dataset
    let dataset = match split {
        Split::Train => train_dataset,
        Split::Val => val_dataset,
        Split::Test => test_dataset,
    };

    dataset.images.push(coco_image);
    dataset.annotations.extend(coco_annotations);

    Ok((annotation_count, skipped_count, invalid_annotations))
}

/// Validate shape points count based on detected input format
fn validate_shape_points(shape: &Shape, input_format: InputAnnotationFormat) -> Result<(), InvalidReason> {
    let points_count = shape.points.len();

    // Empty points is always invalid
    if points_count == 0 {
        return Err(InvalidReason::EmptyPoints);
    }

    // Validate based on detected input format
    match input_format {
        InputAnnotationFormat::Bbox2Point => {
            if points_count != 2 {
                return Err(InvalidReason::PointsCountMismatch {
                    expected_format: input_format,
                    actual_points: points_count,
                });
            }
        }
        InputAnnotationFormat::Bbox4Point => {
            if points_count != 4 {
                return Err(InvalidReason::PointsCountMismatch {
                    expected_format: input_format,
                    actual_points: points_count,
                });
            }
        }
        InputAnnotationFormat::Polygon => {
            if points_count < 3 {
                return Err(InvalidReason::PointsCountMismatch {
                    expected_format: input_format,
                    actual_points: points_count,
                });
            }
        }
        InputAnnotationFormat::Unknown => {
            // For unknown format, just require at least 2 points
            if points_count < 2 {
                return Err(InvalidReason::InsufficientPoints);
            }
        }
    }

    Ok(())
}

/// Convert a LabelMe shape to COCO annotation
fn shape_to_coco_annotation(
    shape: &Shape,
    image_id: u32,
    category_id: u32,
    annotation_id: u32,
    config: &ConversionConfig,
) -> Option<CocoAnnotation> {
    let points = match shape.shape_type.as_str() {
        "polygon" => {
            let mut pts = shape.points.clone();
            // Remove duplicate last point if present
            if pts.len() >= 4 {
                let first = pts[0];
                let last = pts[pts.len() - 1];
                if (first.0 - last.0).abs() < 0.001 && (first.1 - last.1).abs() < 0.001 {
                    pts.pop();
                }
            }
            pts
        }
        "rectangle" => {
            if shape.points.len() < 2 {
                return None;
            }
            rectangle_to_polygon(&shape.points)
        }
        "circle" => {
            if shape.points.len() < 2 {
                return None;
            }
            let (cx, cy) = shape.points[0];
            let (px, py) = shape.points[1];
            let radius = ((cx - px).powi(2) + (cy - py).powi(2)).sqrt();
            crate::labelme_convert::conversion::circle_to_polygon((cx, cy), radius, 12)
        }
        _ => return None,
    };

    if points.len() < 3 {
        return None;
    }

    let area = calculate_polygon_area(&points);
    if area <= 0.0 {
        return None;
    }

    let bbox = calculate_coco_bbox(&points);
    let segmentation = match config.segmentation_mode {
        crate::labelme_convert::config::SegmentationMode::Polygon => {
            Some(vec![flatten_polygon(&points)])
        }
        crate::labelme_convert::config::SegmentationMode::BboxOnly => None,
    };

    Some(CocoAnnotation {
        id: annotation_id,
        image_id,
        category_id,
        bbox,
        area,
        iscrowd: 0,
        segmentation,
    })
}

/// Get the images directory for a given split
fn get_split_images_dir(output_dirs: &CocoOutputDirs, split: Split) -> &Path {
    match split {
        Split::Train => output_dirs.train_images_dir.as_path(),
        Split::Val => output_dirs.val_images_dir.as_path(),
        Split::Test => output_dirs
            .test_images_dir
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or(output_dirs.train_images_dir.as_path()),
    }
}

/// Build COCO categories from label map
fn build_categories(label_map: &HashMap<String, usize>) -> Vec<CocoCategory> {
    let mut sorted: Vec<_> = label_map.iter().collect();
    sorted.sort_by_key(|(_, id)| *id);

    sorted
        .into_iter()
        .map(|(name, id)| CocoCategory {
            id: (*id + 1) as u32, // COCO uses 1-indexed IDs
            name: name.clone(),
            supercategory: "none".to_string(),
        })
        .collect()
}

/// Write COCO dataset to JSON file
fn write_coco_json(path: &Path, dataset: &CocoDataset) -> Result<(), String> {
    let json = serde_json::to_string_pretty(dataset)
        .map_err(|e| format!("Failed to serialize COCO JSON: {}", e))?;

    write_file(path, &json).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coco_dataset_default() {
        let dataset = CocoDataset::default();
        assert_eq!(dataset.categories.len(), 0);
        assert_eq!(dataset.images.len(), 0);
        assert_eq!(dataset.annotations.len(), 0);
    }

    #[test]
    fn test_build_categories() {
        let mut label_map = HashMap::new();
        label_map.insert("cat".to_string(), 0);
        label_map.insert("dog".to_string(), 1);

        let categories = build_categories(&label_map);
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].id, 1);
        assert_eq!(categories[0].name, "cat");
        assert_eq!(categories[1].id, 2);
        assert_eq!(categories[1].name, "dog");
    }
}

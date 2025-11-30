// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::conversion::{determine_split, hash_string, shape_to_yolo_line, Split};
use crate::labelme_convert::io::{
    copy_image, create_dataset_yaml, extract_embedded_image, find_json_files, read_labelme_json,
    resolve_image_path, setup_yolo_directories, write_file,
};
use crate::labelme_convert::types::{
    ConversionResult, InputAnnotationFormat, InvalidAnnotation, ProcessingStats, Shape, YoloOutputDirs,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Main YOLO dataset conversion function
pub fn convert_to_yolo(config: &ConversionConfig) -> ConversionResult {
    // Validate configuration
    if let Err(e) = config.validate() {
        return ConversionResult::failure(vec![e]);
    }

    // Set up output directories
    let output_dirs = match setup_yolo_directories(config) {
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
    let mut skipped_labels: HashSet<String> = HashSet::new();

    // Initialize label map from config if provided
    for (id, label) in config.label_list.iter().enumerate() {
        label_map.insert(label.clone(), id);
    }

    // Find all JSON files
    let json_files = find_json_files(&config.input_dir);
    stats.total_files = json_files.len();

    // If deterministic labels and no predefined list, do a first pass to gather all labels
    if config.deterministic_labels && config.label_list.is_empty() {
        gather_labels(&json_files, &mut label_map);
    }

    // Use the pre-detected input format from config (set by mod.rs convert())
    let input_format = config.detected_input_format.unwrap_or(InputAnnotationFormat::Unknown);

    // Process each JSON file
    for json_path in &json_files {
        match process_single_file(
            json_path,
            config,
            &output_dirs,
            &mut label_map,
            &mut processed_images,
            &mut skipped_labels,
            input_format,
        ) {
            Ok((annotation_count, skipped_count, invalid_list)) => {
                stats.increment_processed();
                stats.add_annotations(annotation_count);
                stats.add_skipped_annotations(skipped_count);
                // Add invalid annotations to stats
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

    // Process background images if enabled
    if config.include_background {
        let bg_files = process_background_images(config, &output_dirs, &processed_images);
        for file_name in bg_files {
            stats.add_background_file(file_name);
        }
    }

    // Update stats with labels
    for label in label_map.keys() {
        stats.add_label(label.clone());
    }

    // Add skipped labels to stats
    for label in skipped_labels {
        stats.add_skipped_label(label);
    }

    // Create dataset.yaml
    if let Err(e) = create_dataset_yaml(&output_dirs.base_dir, &label_map, config.has_test_split())
    {
        errors.push(format!("Failed to create dataset.yaml: {}", e));
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

/// Gather all unique labels from JSON files (first pass for deterministic labeling)
fn gather_labels(json_files: &[std::path::PathBuf], label_map: &mut HashMap<String, usize>) {
    let mut all_labels: HashSet<String> = HashSet::new();

    for json_path in json_files {
        if let Ok(annotation) = read_labelme_json(json_path) {
            for shape in &annotation.shapes {
                all_labels.insert(shape.label.clone());
            }
        }
    }

    // Sort labels alphabetically for deterministic ordering
    let mut sorted_labels: Vec<_> = all_labels.into_iter().collect();
    sorted_labels.sort();

    for (id, label) in sorted_labels.into_iter().enumerate() {
        label_map.insert(label, id);
    }
}

/// Process a single LabelMe JSON file
/// Returns (annotation_count, skipped_count, invalid_annotations)
fn process_single_file(
    json_path: &Path,
    config: &ConversionConfig,
    output_dirs: &YoloOutputDirs,
    label_map: &mut HashMap<String, usize>,
    processed_images: &mut HashSet<String>,
    skipped_labels: &mut HashSet<String>,
    input_format: InputAnnotationFormat,
) -> Result<(usize, usize, Vec<InvalidAnnotation>), String> {
    // Read and parse JSON
    let annotation = read_labelme_json(json_path)?;

    // Resolve image path
    let image_path = resolve_image_path(json_path, &annotation.image_path);

    // Track processed images
    let image_key = image_path.to_string_lossy().to_string();
    processed_images.insert(image_key.clone());

    // Determine split (train/val/test)
    let path_hash = hash_string(&image_key);
    let split = determine_split(path_hash, config.val_size, config.test_size);

    // Get output directories for this split
    let (labels_dir, images_dir) = get_split_dirs(output_dirs, split);

    // Add labels to map if not using predefined list and not deterministic
    if config.label_list.is_empty() && !config.deterministic_labels {
        for shape in &annotation.shapes {
            if !label_map.contains_key(&shape.label) {
                let next_id = label_map.len();
                label_map.insert(shape.label.clone(), next_id);
            }
        }
    }

    // Copy or extract image
    let image_stem = image_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(image_data) = &annotation.image_data {
        // Extract embedded image
        let ext = image_path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "png".to_string());
        let dest_path = images_dir.join(format!("{}.{}", image_stem, ext));
        extract_embedded_image(image_data, &dest_path)?;
    } else if image_path.exists() {
        // Copy image file
        copy_image(&image_path, labels_dir.parent().unwrap().parent().unwrap().join("images").join(split.as_str()).as_path())
            .map_err(|e| format!("Failed to copy image: {}", e))?;
    } else {
        return Err(format!("Image file not found: {}", image_path.display()));
    }

    // Generate YOLO label file
    let mut yolo_lines = Vec::new();
    let mut annotation_count = 0;
    let mut skipped_count = 0;
    let mut invalid_annotations = Vec::new();

    // Get filename for error reporting
    let file_name = json_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    for shape in &annotation.shapes {
        if let Some(class_id) = label_map.get(&shape.label) {
            match shape_to_yolo_line(
                shape,
                *class_id,
                annotation.image_width,
                annotation.image_height,
                config.annotation_format,
                input_format,
            ) {
                Ok(line) => {
                    yolo_lines.push(line);
                    annotation_count += 1;
                }
                Err(reason) => {
                    // Record invalid annotation with reason
                    invalid_annotations.push(InvalidAnnotation {
                        file: file_name.clone(),
                        label: shape.label.clone(),
                        reason: reason.as_str().to_string(),
                        shape_type: shape.shape_type.clone(),
                        points_count: shape.points.len(),
                    });
                    skipped_count += 1;
                }
            }
        } else {
            // Label not in the predefined list, skip it
            // Only record to skipped_labels, NOT to invalid_annotations
            // (because the UI already shows skipped labels separately)
            skipped_labels.insert(shape.label.clone());
            skipped_count += 1;
        }
    }

    // Write label file
    let label_path = labels_dir.join(format!("{}.txt", image_stem));
    let content = yolo_lines.join("\n");
    write_file(&label_path, &content).map_err(|e| format!("Failed to write label file: {}", e))?;

    Ok((annotation_count, skipped_count, invalid_annotations))
}

/// Get the correct directories for a given split
fn get_split_dirs(output_dirs: &YoloOutputDirs, split: Split) -> (&Path, &Path) {
    match split {
        Split::Train => (
            output_dirs.train_labels_dir.as_path(),
            output_dirs.train_images_dir.as_path(),
        ),
        Split::Val => (
            output_dirs.val_labels_dir.as_path(),
            output_dirs.val_images_dir.as_path(),
        ),
        Split::Test => (
            output_dirs
                .test_labels_dir
                .as_ref()
                .map(|p| p.as_path())
                .unwrap_or(output_dirs.train_labels_dir.as_path()),
            output_dirs
                .test_images_dir
                .as_ref()
                .map(|p| p.as_path())
                .unwrap_or(output_dirs.train_images_dir.as_path()),
        ),
    }
}

/// Process background images (images without annotations)
/// Returns the list of background image file names
fn process_background_images(
    config: &ConversionConfig,
    output_dirs: &YoloOutputDirs,
    processed_images: &HashSet<String>,
) -> Vec<String> {
    use crate::labelme_convert::io::find_image_files;

    let image_files = find_image_files(&config.input_dir);
    let mut bg_files = Vec::new();

    for image_path in image_files {
        let image_key = image_path.to_string_lossy().to_string();

        // Skip if already processed
        if processed_images.contains(&image_key) {
            continue;
        }

        // Determine split
        let path_hash = hash_string(&image_key);
        let split = determine_split(path_hash, config.val_size, config.test_size);

        let (labels_dir, images_dir) = get_split_dirs(output_dirs, split);

        // Copy image
        if let Err(e) = copy_image(&image_path, images_dir) {
            eprintln!("Failed to copy background image {}: {}", image_path.display(), e);
            continue;
        }

        // Create empty label file
        let image_stem = image_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let label_path = labels_dir.join(format!("{}.txt", image_stem));

        if let Err(e) = write_file(&label_path, "") {
            eprintln!("Failed to create empty label file: {}", e);
            continue;
        }

        // Get file name for reporting
        let file_name = image_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        bg_files.push(file_name);
    }

    bg_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_gather_labels() {
        let mut label_map = HashMap::new();
        // Empty file list
        gather_labels(&[], &mut label_map);
        assert!(label_map.is_empty());
    }

    #[test]
    fn test_get_split_dirs() {
        let output_dirs = YoloOutputDirs {
            base_dir: PathBuf::from("/test"),
            train_labels_dir: PathBuf::from("/test/labels/train"),
            val_labels_dir: PathBuf::from("/test/labels/val"),
            train_images_dir: PathBuf::from("/test/images/train"),
            val_images_dir: PathBuf::from("/test/images/val"),
            test_labels_dir: Some(PathBuf::from("/test/labels/test")),
            test_images_dir: Some(PathBuf::from("/test/images/test")),
        };

        let (labels, _images) = get_split_dirs(&output_dirs, Split::Train);
        assert_eq!(labels, Path::new("/test/labels/train"));

        let (labels, _images) = get_split_dirs(&output_dirs, Split::Val);
        assert_eq!(labels, Path::new("/test/labels/val"));

        let (labels, _images) = get_split_dirs(&output_dirs, Split::Test);
        assert_eq!(labels, Path::new("/test/labels/test"));
    }
}

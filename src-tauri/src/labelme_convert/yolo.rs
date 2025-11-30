// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::conversion::shape_to_yolo_line;
use crate::labelme_convert::io::{
    copy_image, create_dataset_yaml, extract_embedded_image, find_background_images,
    find_json_files, read_labelme_json, resolve_image_path, setup_yolo_directories, write_file,
};
use crate::labelme_convert::pipeline::{
    determine_split, hash_string, ConversionPipeline, FileType, OutputDirectories,
    ProcessedFileResult, ProcessingContext,
};
use crate::labelme_convert::types::{ConversionResult, InputAnnotationFormat, InvalidAnnotation};
use std::collections::HashSet;
use std::path::Path;

// ============================================================================
// YoloPipeline: ConversionPipeline trait implementation
// ============================================================================

/// YOLO format conversion pipeline
pub struct YoloPipeline;

impl ConversionPipeline for YoloPipeline {
    fn needs_split(&self) -> bool {
        true
    }

    fn setup_output_dirs(
        &self,
        config: &ConversionConfig,
    ) -> Result<Box<dyn OutputDirectories>, String> {
        let dirs = setup_yolo_directories(config)
            .map_err(|e| format!("Failed to create output directories: {}", e))?;
        Ok(Box::new(dirs))
    }

    fn process_file(
        &self,
        json_path: &Path,
        config: &ConversionConfig,
        output_dirs: &dyn OutputDirectories,
        context: &mut ProcessingContext,
    ) -> Result<ProcessedFileResult, String> {
        // Read and parse JSON
        let annotation = read_labelme_json(json_path)?;

        // Resolve image path
        let image_path = resolve_image_path(json_path, &annotation.image_path);
        let image_key = image_path.to_string_lossy().to_string();

        // Check for duplicate processing
        if context.is_image_processed(&image_key) {
            return Ok(ProcessedFileResult::default());
        }
        context.mark_image_processed(image_key.clone());

        // Determine split (train/val/test)
        let path_hash = hash_string(&image_key);
        let split = determine_split(path_hash, config.val_size, config.test_size);

        // Get output directories for this split
        let labels_dir = output_dirs.get_output_dir(split, FileType::Label);
        let images_dir = output_dirs.get_output_dir(split, FileType::Image);

        // Add labels to map if not using predefined list and not deterministic
        if config.label_list.is_empty() && !config.deterministic_labels {
            for shape in &annotation.shapes {
                context.ensure_label(&shape.label);
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
            copy_image(&image_path, images_dir)
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

        // Get input format from config
        let input_format = config
            .detected_input_format
            .unwrap_or(InputAnnotationFormat::Unknown);

        for shape in &annotation.shapes {
            if let Some(&class_id) = context.label_map.get(&shape.label) {
                match shape_to_yolo_line(
                    shape,
                    class_id,
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
                        invalid_annotations.push(InvalidAnnotation {
                            file: file_name.clone(),
                            label: shape.label.clone(),
                            reason: reason.as_str(),
                            shape_type: shape.shape_type.clone(),
                            points_count: shape.points.len(),
                        });
                        skipped_count += 1;
                    }
                }
            } else {
                // Label not in the predefined list
                context.add_skipped_label(&shape.label);
                skipped_count += 1;
            }
        }

        // Write label file
        let label_path = labels_dir.join(format!("{}.txt", image_stem));
        let content = yolo_lines.join("\n");
        write_file(&label_path, &content)
            .map_err(|e| format!("Failed to write label file: {}", e))?;

        Ok(ProcessedFileResult {
            annotations_processed: annotation_count,
            annotations_skipped: skipped_count,
            invalid_annotations,
        })
    }

    fn finalize(
        &self,
        config: &ConversionConfig,
        output_dirs: &dyn OutputDirectories,
        context: &ProcessingContext,
    ) -> Result<(), String> {
        // Create dataset.yaml
        create_dataset_yaml(output_dirs.base_dir(), &context.label_map, config.has_test_split())
            .map_err(|e| format!("Failed to create dataset.yaml: {}", e))?;
        Ok(())
    }
}

// ============================================================================
// Public conversion function (backward compatible entry point)
// ============================================================================

/// Main YOLO dataset conversion function
pub fn convert_to_yolo(config: &ConversionConfig) -> ConversionResult {
    // Validate configuration
    if let Err(e) = config.validate() {
        return ConversionResult::failure(vec![e]);
    }

    let pipeline = YoloPipeline;

    // Set up output directories
    let output_dirs = match pipeline.setup_output_dirs(config) {
        Ok(dirs) => dirs,
        Err(e) => return ConversionResult::failure(vec![e]),
    };

    // Initialize processing context
    let mut context = if config.label_list.is_empty() {
        ProcessingContext::new()
    } else {
        ProcessingContext::with_labels(&config.label_list)
    };

    // Find all JSON files
    let json_files = find_json_files(&config.input_dir);
    context.stats.total_files = json_files.len();

    // If deterministic labels and no predefined list, gather all labels first
    if config.deterministic_labels && config.label_list.is_empty() {
        pipeline.gather_labels(&json_files, &mut context);
    }

    // Process each JSON file
    for json_path in &json_files {
        match pipeline.process_file(json_path, config, output_dirs.as_ref(), &mut context) {
            Ok(result) => {
                context.stats.increment_processed();
                context.stats.add_annotations(result.annotations_processed);
                context.stats.add_skipped_annotations(result.annotations_skipped);
                for invalid in result.invalid_annotations {
                    context.stats.add_invalid_annotation(invalid);
                }
            }
            Err(e) => {
                context.stats.increment_failed();
                context.add_error(format!("{}: {}", json_path.display(), e));
            }
        }
    }

    // Process background images if enabled
    if config.include_background {
        let bg_files =
            process_background_images(config, output_dirs.as_ref(), &context.processed_images);
        for file_name in bg_files {
            context.stats.add_background_file(file_name);
        }
    }

    // Update stats with labels
    for label in context.label_map.keys() {
        context.stats.add_label(label.clone());
    }

    // Add skipped labels to stats
    for label in &context.skipped_labels {
        context.stats.add_skipped_label(label.clone());
    }

    // Finalize (create dataset.yaml)
    if let Err(e) = pipeline.finalize(config, output_dirs.as_ref(), &context) {
        context.add_error(e);
    }

    if context.errors.is_empty() {
        ConversionResult::success(
            output_dirs.base_dir().to_string_lossy().to_string(),
            context.stats,
        )
    } else {
        let mut result = ConversionResult::success(
            output_dirs.base_dir().to_string_lossy().to_string(),
            context.stats,
        );
        result.errors = context.errors;
        result
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Process background images (images without annotations)
/// Returns the list of background image file names
fn process_background_images(
    config: &ConversionConfig,
    output_dirs: &dyn OutputDirectories,
    processed_images: &HashSet<String>,
) -> Vec<String> {
    let bg_images = find_background_images(&config.input_dir, processed_images);
    let mut bg_files = Vec::new();

    for image_path in bg_images {
        let image_key = image_path.to_string_lossy().to_string();

        // Determine split
        let path_hash = hash_string(&image_key);
        let split = determine_split(path_hash, config.val_size, config.test_size);

        let labels_dir = output_dirs.get_output_dir(split, FileType::Label);
        let images_dir = output_dirs.get_output_dir(split, FileType::Image);

        // Copy image
        if let Err(e) = copy_image(&image_path, images_dir) {
            eprintln!(
                "Failed to copy background image {}: {}",
                image_path.display(),
                e
            );
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
    use crate::labelme_convert::pipeline::Split;
    use crate::labelme_convert::types::YoloOutputDirs;
    use std::path::PathBuf;

    #[test]
    fn test_yolo_pipeline_needs_split() {
        let pipeline = YoloPipeline;
        assert!(pipeline.needs_split());
    }

    #[test]
    fn test_yolo_output_dirs() {
        let output_dirs = YoloOutputDirs {
            base_dir: PathBuf::from("/test"),
            train_labels_dir: PathBuf::from("/test/labels/train"),
            val_labels_dir: PathBuf::from("/test/labels/val"),
            train_images_dir: PathBuf::from("/test/images/train"),
            val_images_dir: PathBuf::from("/test/images/val"),
            test_labels_dir: Some(PathBuf::from("/test/labels/test")),
            test_images_dir: Some(PathBuf::from("/test/images/test")),
        };

        assert_eq!(
            output_dirs.get_output_dir(Split::Train, FileType::Label),
            Path::new("/test/labels/train")
        );
        assert_eq!(
            output_dirs.get_output_dir(Split::Val, FileType::Image),
            Path::new("/test/images/val")
        );
        assert_eq!(
            output_dirs.get_output_dir(Split::Test, FileType::Label),
            Path::new("/test/labels/test")
        );
    }
}

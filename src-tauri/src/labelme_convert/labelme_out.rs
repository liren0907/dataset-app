//! LabelMe to LabelMe conversion
//!
//! This module handles LabelMe → LabelMe conversion, which is useful for:
//! - Filtering labels (keeping only selected labels)
//! - Reordering labels (according to a predefined list)
//! - Removing embedded imageData to reduce file size
//! - Copying only annotated images to a new location
//!
//! Unlike YOLO/COCO conversion, this does NOT split the dataset into
//! train/val/test sets.

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::io::{
    copy_image, find_json_files, read_labelme_json, resolve_image_path,
    setup_labelme_directories, write_labelme_json,
};
use crate::labelme_convert::pipeline::{
    ConversionPipeline, FileType, OutputDirectories, ProcessedFileResult, ProcessingContext, Split,
};
use crate::labelme_convert::types::{
    ConversionResult, LabelMeAnnotation, LabelMeOutputDirs, ProcessingStats, Shape,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// LabelMe to LabelMe conversion pipeline
pub struct LabelMePipeline;

impl ConversionPipeline for LabelMePipeline {
    fn needs_split(&self) -> bool {
        false
    }

    fn setup_output_dirs(
        &self,
        config: &ConversionConfig,
    ) -> Result<Box<dyn OutputDirectories>, String> {
        let dirs = setup_labelme_directories(config)
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
        // Read the original LabelMe JSON
        let mut annotation = read_labelme_json(json_path)?;

        // Resolve image path
        let image_path = resolve_image_path(json_path, &annotation.image_path);
        let image_key = image_path.to_string_lossy().to_string();

        // Check for duplicate processing
        if context.is_image_processed(&image_key) {
            return Ok(ProcessedFileResult::default());
        }
        context.mark_image_processed(image_key);

        // Filter shapes based on label list
        let (filtered_shapes, skipped_count) = filter_shapes(
            &annotation.shapes,
            &config.label_list,
            context,
        );

        // Update annotation with filtered shapes
        annotation.shapes = filtered_shapes;

        // Remove imageData if configured
        if config.remove_image_data {
            annotation.image_data = None;
        }

        // Get output directory (no split for LabelMe)
        let output_dir = output_dirs.get_output_dir(Split::None, FileType::Annotation);

        // Get the JSON filename
        let json_filename = json_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.json".to_string());

        // Write the new LabelMe JSON
        let output_json_path = output_dir.join(&json_filename);
        write_labelme_json(&output_json_path, &annotation)?;

        // Copy the image file (if it exists and imageData is not embedded)
        if image_path.exists() {
            copy_image(&image_path, output_dir)
                .map_err(|e| format!("Failed to copy image: {}", e))?;
        }

        let annotations_processed = annotation.shapes.len();

        Ok(ProcessedFileResult {
            annotations_processed,
            annotations_skipped: skipped_count,
            invalid_annotations: Vec::new(), // LabelMe doesn't have "invalid" annotations
        })
    }

    fn finalize(
        &self,
        config: &ConversionConfig,
        output_dirs: &dyn OutputDirectories,
        context: &ProcessingContext,
    ) -> Result<(), String> {
        // Optionally create a labels.txt file listing all labels
        if !context.label_map.is_empty() {
            let labels_path = output_dirs.base_dir().join("labels.txt");

            // Sort labels by ID
            let mut sorted_labels: Vec<_> = context.label_map.iter().collect();
            sorted_labels.sort_by_key(|(_, id)| *id);

            let content = sorted_labels
                .iter()
                .map(|(label, _)| label.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            std::fs::write(&labels_path, content)
                .map_err(|e| format!("Failed to write labels.txt: {}", e))?;
        }

        // Create a summary file
        let summary_path = output_dirs.base_dir().join("conversion_summary.txt");
        let summary = format!(
            "LabelMe Conversion Summary\n\
             ==========================\n\
             Source: {}\n\
             Files processed: {}\n\
             Total annotations: {}\n\
             Skipped annotations: {}\n\
             Labels: {}\n",
            config.input_dir.display(),
            context.stats.processed_files,
            context.stats.total_annotations,
            context.stats.skipped_annotations,
            context.label_map.keys().cloned().collect::<Vec<_>>().join(", ")
        );

        std::fs::write(&summary_path, summary)
            .map_err(|e| format!("Failed to write summary: {}", e))?;

        Ok(())
    }
}

/// Filter shapes based on label list
///
/// Returns (filtered_shapes, skipped_count)
fn filter_shapes(
    shapes: &[Shape],
    label_list: &[String],
    context: &mut ProcessingContext,
) -> (Vec<Shape>, usize) {
    // If label list is empty, keep all shapes
    if label_list.is_empty() {
        // Still need to add labels to context
        for shape in shapes {
            context.ensure_label(&shape.label);
        }
        return (shapes.to_vec(), 0);
    }

    // Create a set for fast lookup
    let allowed_labels: HashSet<&str> = label_list.iter().map(|s| s.as_str()).collect();

    let mut filtered = Vec::new();
    let mut skipped = 0;

    for shape in shapes {
        if allowed_labels.contains(shape.label.as_str()) {
            context.ensure_label(&shape.label);
            filtered.push(shape.clone());
        } else {
            context.add_skipped_label(&shape.label);
            skipped += 1;
        }
    }

    (filtered, skipped)
}

/// Main conversion function for LabelMe → LabelMe
pub fn convert_to_labelme(config: &ConversionConfig) -> ConversionResult {
    // Validate configuration
    if let Err(e) = config.validate() {
        return ConversionResult::failure(vec![e]);
    }

    let pipeline = LabelMePipeline;

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

    // Process each JSON file
    for json_path in &json_files {
        match pipeline.process_file(json_path, config, output_dirs.as_ref(), &mut context) {
            Ok(result) => {
                context.stats.increment_processed();
                context.stats.add_annotations(result.annotations_processed);
                context.stats.add_skipped_annotations(result.annotations_skipped);
            }
            Err(e) => {
                context.stats.increment_failed();
                context.add_error(format!("{}: {}", json_path.display(), e));
            }
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

    // Finalize
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_shapes_empty_list() {
        let shapes = vec![
            Shape {
                label: "cat".to_string(),
                points: vec![(0.0, 0.0), (10.0, 10.0)],
                group_id: None,
                shape_type: "rectangle".to_string(),
                description: None,
                mask: None,
                flags: None,
            },
            Shape {
                label: "dog".to_string(),
                points: vec![(0.0, 0.0), (10.0, 10.0)],
                group_id: None,
                shape_type: "rectangle".to_string(),
                description: None,
                mask: None,
                flags: None,
            },
        ];

        let mut context = ProcessingContext::new();
        let (filtered, skipped) = filter_shapes(&shapes, &[], &mut context);

        assert_eq!(filtered.len(), 2);
        assert_eq!(skipped, 0);
        assert!(context.label_map.contains_key("cat"));
        assert!(context.label_map.contains_key("dog"));
    }

    #[test]
    fn test_filter_shapes_with_list() {
        let shapes = vec![
            Shape {
                label: "cat".to_string(),
                points: vec![(0.0, 0.0), (10.0, 10.0)],
                group_id: None,
                shape_type: "rectangle".to_string(),
                description: None,
                mask: None,
                flags: None,
            },
            Shape {
                label: "dog".to_string(),
                points: vec![(0.0, 0.0), (10.0, 10.0)],
                group_id: None,
                shape_type: "rectangle".to_string(),
                description: None,
                mask: None,
                flags: None,
            },
        ];

        let label_list = vec!["cat".to_string()];
        let mut context = ProcessingContext::new();
        let (filtered, skipped) = filter_shapes(&shapes, &label_list, &mut context);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].label, "cat");
        assert_eq!(skipped, 1);
        assert!(context.label_map.contains_key("cat"));
        assert!(!context.label_map.contains_key("dog"));
        assert!(context.skipped_labels.contains("dog"));
    }

    #[test]
    fn test_pipeline_needs_split() {
        let pipeline = LabelMePipeline;
        assert!(!pipeline.needs_split());
    }
}

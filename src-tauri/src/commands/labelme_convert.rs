// Tauri commands for LabelMe format conversion
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::{
    convert, AnnotationFormat, ConversionConfig, ConversionResult, LabelMeOutputFormat,
    OutputFormat, SegmentationMode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request parameters for LabelMe conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertLabelMeRequest {
    /// Input directory containing LabelMe JSON files
    pub input_dir: String,

    /// Output directory (optional, defaults to input_dir)
    #[serde(default)]
    pub output_dir: Option<String>,

    /// Custom dataset folder name (optional, uses generated name if empty)
    #[serde(default)]
    pub custom_dataset_name: Option<String>,

    /// Output format: "yolo" or "coco"
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Annotation format: "bbox" or "polygon" (for YOLO)
    #[serde(default = "default_annotation_format")]
    pub annotation_format: String,

    /// Validation set proportion (0.0 - 1.0)
    #[serde(default = "default_val_size")]
    pub val_size: f32,

    /// Test set proportion (0.0 - 1.0)
    #[serde(default)]
    pub test_size: f32,

    /// Random seed for reproducible splits
    #[serde(default = "default_seed")]
    pub seed: u64,

    /// Include images without annotations as background
    #[serde(default)]
    pub include_background: bool,

    /// Predefined label list (empty = auto-detect)
    #[serde(default)]
    pub label_list: Vec<String>,

    /// Enable deterministic label ID assignment
    #[serde(default)]
    pub deterministic_labels: bool,

    /// Segmentation mode for COCO: "polygon" or "bbox_only"
    #[serde(default = "default_segmentation_mode")]
    pub segmentation_mode: String,

    // LabelMe-specific options
    /// Remove imageData from output JSON (for LabelMe output)
    #[serde(default)]
    pub remove_image_data: bool,

    /// LabelMe output point format: "original", "bbox_2point", or "bbox_4point"
    #[serde(default = "default_labelme_output_format")]
    pub labelme_output_format: String,
}

fn default_output_format() -> String {
    "yolo".to_string()
}

fn default_annotation_format() -> String {
    "bbox".to_string()
}

fn default_val_size() -> f32 {
    0.2
}

fn default_seed() -> u64 {
    42
}

fn default_segmentation_mode() -> String {
    "polygon".to_string()
}

fn default_labelme_output_format() -> String {
    "original".to_string()
}

impl ConvertLabelMeRequest {
    /// Convert request to internal ConversionConfig
    pub fn to_config(&self) -> Result<ConversionConfig, String> {
        let output_format = match self.output_format.to_lowercase().as_str() {
            "yolo" => OutputFormat::Yolo,
            "coco" => OutputFormat::Coco,
            "labelme" => OutputFormat::LabelMe,
            other => return Err(format!("Unknown output format: {}", other)),
        };

        let annotation_format = match self.annotation_format.to_lowercase().as_str() {
            "bbox" => AnnotationFormat::Bbox,
            "polygon" => AnnotationFormat::Polygon,
            other => return Err(format!("Unknown annotation format: {}", other)),
        };

        let segmentation_mode = match self.segmentation_mode.to_lowercase().as_str() {
            "polygon" => SegmentationMode::Polygon,
            "bbox_only" | "bboxonly" => SegmentationMode::BboxOnly,
            other => return Err(format!("Unknown segmentation mode: {}", other)),
        };

        let mut config = ConversionConfig::new(PathBuf::from(&self.input_dir))
            .with_output_format(output_format)
            .with_annotation_format(annotation_format)
            .with_val_size(self.val_size)
            .with_test_size(self.test_size)
            .with_seed(self.seed)
            .with_background(self.include_background)
            .with_labels(self.label_list.clone())
            .with_custom_name(self.custom_dataset_name.clone());

        config.deterministic_labels = self.deterministic_labels;
        config.segmentation_mode = segmentation_mode;

        // LabelMe-specific options
        if output_format == OutputFormat::LabelMe {
            config.skip_split = true; // LabelMe output never uses splits
            config.remove_image_data = self.remove_image_data;

            // Parse LabelMe output format
            config.labelme_output_format = match self.labelme_output_format.to_lowercase().as_str()
            {
                "original" => LabelMeOutputFormat::Original,
                "bbox_2point" | "bbox2point" => LabelMeOutputFormat::Bbox2Point,
                "bbox_4point" | "bbox4point" => LabelMeOutputFormat::Bbox4Point,
                other => return Err(format!("Unknown LabelMe output format: {}", other)),
            };
        }

        if let Some(output_dir) = &self.output_dir {
            config = config.with_output_dir(PathBuf::from(output_dir));
        }

        Ok(config)
    }
}

/// Convert LabelMe annotations to YOLO or COCO format
///
/// This command takes a request object with all conversion parameters
/// and returns a result containing success status, output directory,
/// processing statistics, and any errors.
#[tauri::command]
pub fn convert_labelme(request: ConvertLabelMeRequest) -> Result<ConversionResult, String> {
    let config = request.to_config()?;

    // Validate configuration
    config.validate()?;

    // Run conversion
    Ok(convert(&config))
}

/// Quick conversion to YOLO format with default settings
///
/// This is a convenience command for simple YOLO conversion
/// with automatic train/val splitting.
#[tauri::command]
pub fn quick_convert_to_yolo(
    input_dir: String,
    val_size: Option<f32>,
    use_polygon: Option<bool>,
) -> Result<ConversionResult, String> {
    let config = ConversionConfig::new(PathBuf::from(&input_dir))
        .with_output_format(OutputFormat::Yolo)
        .with_annotation_format(if use_polygon.unwrap_or(false) {
            AnnotationFormat::Polygon
        } else {
            AnnotationFormat::Bbox
        })
        .with_val_size(val_size.unwrap_or(0.2));

    config.validate()?;

    Ok(convert(&config))
}

/// Quick conversion to COCO format with default settings
#[tauri::command]
pub fn quick_convert_to_coco(
    input_dir: String,
    val_size: Option<f32>,
) -> Result<ConversionResult, String> {
    let config = ConversionConfig::new(PathBuf::from(&input_dir))
        .with_output_format(OutputFormat::Coco)
        .with_val_size(val_size.unwrap_or(0.2));

    config.validate()?;

    Ok(convert(&config))
}

/// Get available labels from a directory of LabelMe JSON files
///
/// This command scans all JSON files in the directory and extracts
/// all unique label names found in the annotations.
#[tauri::command]
pub fn scan_labelme_labels(input_dir: String) -> Result<Vec<String>, String> {
    use crate::labelme_convert::io::{find_json_files, read_labelme_json};
    use std::collections::HashSet;

    let input_path = PathBuf::from(&input_dir);

    if !input_path.exists() {
        return Err(format!("Directory does not exist: {}", input_dir));
    }

    let json_files = find_json_files(&input_path);
    let mut labels: HashSet<String> = HashSet::new();

    for json_path in json_files {
        if let Ok(annotation) = read_labelme_json(&json_path) {
            for shape in annotation.shapes {
                labels.insert(shape.label);
            }
        }
    }

    let mut sorted_labels: Vec<_> = labels.into_iter().collect();
    sorted_labels.sort();

    Ok(sorted_labels)
}

/// Count LabelMe JSON files in a directory
#[tauri::command]
pub fn count_labelme_files(input_dir: String) -> Result<usize, String> {
    use crate::labelme_convert::io::find_json_files;

    let input_path = PathBuf::from(&input_dir);

    if !input_path.exists() {
        return Err(format!("Directory does not exist: {}", input_dir));
    }

    let json_files = find_json_files(&input_path);
    Ok(json_files.len())
}

/// Scan labels with occurrence counts from a directory of LabelMe JSON files
///
/// This command scans all JSON files and returns each unique label with its count.
/// Useful for displaying label statistics in the UI.
#[tauri::command]
pub fn scan_labelme_labels_with_counts(
    input_dir: String,
) -> Result<std::collections::HashMap<String, usize>, String> {
    use crate::labelme_convert::io::{find_json_files, read_labelme_json};
    use std::collections::HashMap;

    let input_path = PathBuf::from(&input_dir);

    if !input_path.exists() {
        return Err(format!("Directory does not exist: {}", input_dir));
    }

    let json_files = find_json_files(&input_path);
    let mut label_counts: HashMap<String, usize> = HashMap::new();

    for json_path in json_files {
        if let Ok(annotation) = read_labelme_json(&json_path) {
            for shape in annotation.shapes {
                *label_counts.entry(shape.label).or_insert(0) += 1;
            }
        }
    }

    Ok(label_counts)
}

/// Analyze a LabelMe dataset to detect input annotation format
///
/// This command analyzes the dataset to determine:
/// - Detected input format (2-point bbox, 4-point bbox, or polygon)
/// - Confidence score for the detection
/// - Points distribution statistics
/// - Human-readable format description
///
/// Useful for displaying dataset analysis in the UI before conversion.
#[tauri::command]
pub fn analyze_labelme_dataset(input_dir: String) -> Result<DatasetAnalysisResponse, String> {
    use crate::labelme_convert::detection::analyze_dataset;

    let input_path = PathBuf::from(&input_dir);

    if !input_path.exists() {
        return Err(format!("Directory does not exist: {}", input_dir));
    }

    let analysis = analyze_dataset(&input_path);

    Ok(DatasetAnalysisResponse {
        input_format: format!("{:?}", analysis.input_format),
        total_files: analysis.total_files,
        sample_files: analysis.sample_files,
        sample_annotations: analysis.sample_annotations,
        confidence: analysis.confidence,
        confidence_percent: format!("{:.1}%", analysis.confidence * 100.0),
        points_distribution: analysis.points_distribution,
        format_description: analysis.format_description,
    })
}

/// Response structure for dataset analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetAnalysisResponse {
    /// Detected input format as string: "Bbox2Point", "Bbox4Point", "Polygon", "Unknown"
    pub input_format: String,
    /// Total number of JSON files in the dataset
    pub total_files: usize,
    /// Number of files sampled for analysis
    pub sample_files: usize,
    /// Total number of annotations sampled
    pub sample_annotations: usize,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Confidence as percentage string (e.g., "87.5%")
    pub confidence_percent: String,
    /// Distribution of point counts
    pub points_distribution: std::collections::HashMap<usize, usize>,
    /// Human-readable description of the detected format
    pub format_description: String,
}

// ===== 🆕 Async commands with progress reporting =====

/// Asynchronously scan labels with progress updates
///
/// This is the new async version that doesn't block the UI.
/// It reports progress via the "label-scan-progress" event.
#[tauri::command]
pub async fn scan_labelme_labels_async(
    window: tauri::Window,
    input_dir: String,
) -> Result<Vec<String>, String> {
    use crate::labelme_convert::{progress::ProgressEmitter, scanner};
    use std::path::PathBuf;

    let input_path = PathBuf::from(&input_dir);
    let progress = ProgressEmitter::new(window, "label-scan-progress");

    scanner::scan_labels_async(input_path, Some(progress)).await
}

/// Asynchronously scan labels with counts and progress updates
///
/// Reports progress via the "count-scan-progress" event.
#[tauri::command]
pub async fn scan_labelme_labels_with_counts_async(
    window: tauri::Window,
    input_dir: String,
) -> Result<std::collections::HashMap<String, usize>, String> {
    use crate::labelme_convert::{progress::ProgressEmitter, scanner};
    use std::path::PathBuf;

    let input_path = PathBuf::from(&input_dir);
    let progress = ProgressEmitter::new(window, "count-scan-progress");

    scanner::scan_labels_with_counts_async(input_path, Some(progress)).await
}

/// Asynchronously analyze dataset format with progress updates
///
/// Reports progress via the "format-analysis-progress" event.
#[tauri::command]
pub async fn analyze_labelme_dataset_async(
    window: tauri::Window,
    input_dir: String,
) -> Result<DatasetAnalysisResponse, String> {
    use crate::labelme_convert::{progress::ProgressEmitter, scanner};
    use std::path::PathBuf;

    let input_path = PathBuf::from(&input_dir);
    let progress = ProgressEmitter::new(window, "format-analysis-progress");

    let analysis = scanner::analyze_dataset_async(input_path, Some(progress)).await?;

    Ok(DatasetAnalysisResponse {
        input_format: format!("{:?}", analysis.input_format),
        total_files: analysis.total_files,
        sample_files: analysis.sample_files,
        sample_annotations: analysis.sample_annotations,
        confidence: analysis.confidence,
        confidence_percent: format!("{:.1}%", analysis.confidence * 100.0),
        points_distribution: analysis.points_distribution,
        format_description: analysis.format_description,
    })
}

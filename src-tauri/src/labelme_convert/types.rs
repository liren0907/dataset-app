// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported image formats
pub const IMG_FORMATS: &[&str] = &[
    "bmp", "dng", "jpeg", "jpg", "mpo", "png", "tif", "tiff", "webp", "pfm",
];

/// Shape annotation in LabelMe format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Shape {
    pub label: String,
    pub points: Vec<(f64, f64)>,
    pub group_id: Option<i64>,
    pub shape_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub mask: Option<String>,
    #[serde(default)]
    pub flags: Option<HashMap<String, bool>>,
}

/// Complete LabelMe annotation structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LabelMeAnnotation {
    pub version: String,
    #[serde(default)]
    pub flags: Option<HashMap<String, bool>>,
    pub shapes: Vec<Shape>,
    pub image_path: String,
    #[serde(default)]
    pub image_data: Option<String>,
    pub image_height: u32,
    pub image_width: u32,
}

/// Output directories for YOLO dataset
#[derive(Debug, Clone)]
pub struct YoloOutputDirs {
    pub base_dir: PathBuf,
    pub train_labels_dir: PathBuf,
    pub val_labels_dir: PathBuf,
    pub train_images_dir: PathBuf,
    pub val_images_dir: PathBuf,
    pub test_labels_dir: Option<PathBuf>,
    pub test_images_dir: Option<PathBuf>,
}

/// Output directories for COCO dataset
#[derive(Debug, Clone)]
pub struct CocoOutputDirs {
    pub base_dir: PathBuf,
    pub annotations_dir: PathBuf,
    pub train_images_dir: PathBuf,
    pub val_images_dir: PathBuf,
    pub test_images_dir: Option<PathBuf>,
}

/// Output directories for LabelMe dataset (no split)
#[derive(Debug, Clone)]
pub struct LabelMeOutputDirs {
    pub base_dir: PathBuf,
    /// Single output directory for all files
    pub output_dir: PathBuf,
}

/// Split data containers
#[derive(Debug, Default)]
pub struct SplitData {
    pub train_files: Vec<PathBuf>,
    pub val_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
}

/// Invalid annotation record with reason
#[derive(Debug, Clone, Serialize)]
pub struct InvalidAnnotation {
    pub file: String,
    pub label: String,
    pub reason: String,
    pub shape_type: String,
    pub points_count: usize,
}

/// Detected input annotation format based on sampling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputAnnotationFormat {
    /// 2-point bounding box (rectangle with 2 corner points)
    Bbox2Point,
    /// 4-point bounding box (rectangle with 4 corner points)
    Bbox4Point,
    /// Polygon with variable points (>= 3)
    Polygon,
    /// Mixed or unknown format
    Unknown,
}

impl InputAnnotationFormat {
    pub fn expected_points_description(&self) -> &'static str {
        match self {
            InputAnnotationFormat::Bbox2Point => "需要 2 個點",
            InputAnnotationFormat::Bbox4Point => "需要 4 個點",
            InputAnnotationFormat::Polygon => "需要至少 3 個點",
            InputAnnotationFormat::Unknown => "格式未知",
        }
    }
}

/// Invalid annotation reason types
#[derive(Debug, Clone, Copy)]
pub enum InvalidReason {
    EmptyPoints,
    ZeroArea,
    InsufficientPoints,
    LabelNotInList,
    /// Points count doesn't match expected format (e.g., polygon with only 2 points)
    PointsCountMismatch {
        expected_format: InputAnnotationFormat,
        actual_points: usize,
    },
}

impl InvalidReason {
    pub fn as_str(&self) -> String {
        match self {
            InvalidReason::EmptyPoints => "標註點為空".to_string(),
            InvalidReason::ZeroArea => "標註面積為零（width 或 height <= 0）".to_string(),
            InvalidReason::InsufficientPoints => "多邊形點數不足（需要至少 3 個點）".to_string(),
            InvalidReason::LabelNotInList => "標籤不在選定列表中".to_string(),
            InvalidReason::PointsCountMismatch { expected_format, actual_points } => {
                format!(
                    "點數不符合資料集格式（{}，實際 {} 個點）",
                    expected_format.expected_points_description(),
                    actual_points
                )
            }
        }
    }
}

/// Processing statistics
#[derive(Debug, Default, Clone, Serialize)]
pub struct ProcessingStats {
    pub total_files: usize,
    pub processed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub total_annotations: usize,
    pub skipped_annotations: usize,
    /// Background images (images without any JSON annotation file)
    pub background_images: usize,
    /// Background image file names (limited to first 100)
    pub background_files: Vec<String>,
    /// Images that became empty after label filtering
    pub filtered_empty_images: usize,
    /// Filtered empty image file names (limited to first 100)
    pub filtered_empty_files: Vec<String>,
    pub labels_found: Vec<String>,
    pub skipped_labels: Vec<String>,
    /// Detailed invalid annotation records (limited to first 100)
    pub invalid_annotations: Vec<InvalidAnnotation>,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_total(&mut self) {
        self.total_files += 1;
    }

    pub fn increment_processed(&mut self) {
        self.processed_files += 1;
    }

    pub fn increment_skipped(&mut self) {
        self.skipped_files += 1;
    }

    pub fn increment_failed(&mut self) {
        self.failed_files += 1;
    }

    pub fn add_annotations(&mut self, count: usize) {
        self.total_annotations += count;
    }

    pub fn add_skipped_annotations(&mut self, count: usize) {
        self.skipped_annotations += count;
    }

    pub fn increment_background(&mut self) {
        self.background_images += 1;
    }

    /// Add a background file name (limited to first 100 to avoid memory issues)
    pub fn add_background_file(&mut self, file_name: String) {
        self.background_images += 1;
        if self.background_files.len() < 100 {
            self.background_files.push(file_name);
        }
    }

    /// Add a filtered empty file name (limited to first 100 to avoid memory issues)
    /// These are images that had annotations but all labels were filtered out
    pub fn add_filtered_empty_file(&mut self, file_name: String) {
        self.filtered_empty_images += 1;
        if self.filtered_empty_files.len() < 100 {
            self.filtered_empty_files.push(file_name);
        }
    }

    pub fn add_label(&mut self, label: String) {
        if !self.labels_found.contains(&label) {
            self.labels_found.push(label);
        }
    }

    pub fn add_skipped_label(&mut self, label: String) {
        if !self.skipped_labels.contains(&label) {
            self.skipped_labels.push(label);
        }
    }

    /// Add an invalid annotation record (limited to first 100 to avoid memory issues)
    /// Note: Only real errors should be added here, not label filtering skips
    pub fn add_invalid_annotation(&mut self, annotation: InvalidAnnotation) {
        if self.invalid_annotations.len() < 100 {
            self.invalid_annotations.push(annotation);
        }
    }
}

/// Progress callback type for reporting conversion progress
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// Conversion result
#[derive(Debug, Clone, Serialize)]
pub struct ConversionResult {
    pub success: bool,
    pub output_dir: String,
    pub stats: ProcessingStats,
    pub errors: Vec<String>,
}

impl ConversionResult {
    pub fn success(output_dir: String, stats: ProcessingStats) -> Self {
        Self {
            success: true,
            output_dir,
            stats,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            output_dir: String::new(),
            stats: ProcessingStats::default(),
            errors,
        }
    }
}

/// Check if a file extension is a supported image format
pub fn is_image_extension(ext: &str) -> bool {
    IMG_FORMATS.contains(&ext.to_lowercase().as_str())
}

// ============================================================================
// OutputDirectories trait implementations
// ============================================================================

use crate::labelme_convert::pipeline::{FileType, OutputDirectories, Split};

impl OutputDirectories for YoloOutputDirs {
    fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
    }

    fn get_output_dir(&self, split: Split, file_type: FileType) -> &std::path::Path {
        match (split, file_type) {
            (Split::Train, FileType::Image) => &self.train_images_dir,
            (Split::Train, FileType::Label) => &self.train_labels_dir,
            (Split::Val, FileType::Image) => &self.val_images_dir,
            (Split::Val, FileType::Label) => &self.val_labels_dir,
            (Split::Test, FileType::Image) => {
                self.test_images_dir.as_ref().unwrap_or(&self.train_images_dir)
            }
            (Split::Test, FileType::Label) => {
                self.test_labels_dir.as_ref().unwrap_or(&self.train_labels_dir)
            }
            // YOLO doesn't have separate annotation files
            (_, FileType::Annotation) => &self.base_dir,
            // Default to train for Split::None
            (Split::None, FileType::Image) => &self.train_images_dir,
            (Split::None, FileType::Label) => &self.train_labels_dir,
        }
    }

    fn uses_splits(&self) -> bool {
        true
    }
}

impl OutputDirectories for CocoOutputDirs {
    fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
    }

    fn get_output_dir(&self, split: Split, file_type: FileType) -> &std::path::Path {
        match (split, file_type) {
            (Split::Train, FileType::Image) => &self.train_images_dir,
            (Split::Val, FileType::Image) => &self.val_images_dir,
            (Split::Test, FileType::Image) => {
                self.test_images_dir.as_ref().unwrap_or(&self.train_images_dir)
            }
            (_, FileType::Annotation) => &self.annotations_dir,
            // COCO doesn't have separate label files (annotations are in JSON)
            (_, FileType::Label) => &self.annotations_dir,
            // Default to train for Split::None
            (Split::None, FileType::Image) => &self.train_images_dir,
        }
    }

    fn uses_splits(&self) -> bool {
        true
    }
}

impl OutputDirectories for LabelMeOutputDirs {
    fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
    }

    fn get_output_dir(&self, _split: Split, _file_type: FileType) -> &std::path::Path {
        // LabelMe output doesn't use splits - everything goes to one directory
        &self.output_dir
    }

    fn uses_splits(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_image_extension() {
        assert!(is_image_extension("jpg"));
        assert!(is_image_extension("JPG"));
        assert!(is_image_extension("png"));
        assert!(!is_image_extension("txt"));
        assert!(!is_image_extension("json"));
    }

    #[test]
    fn test_labelme_json_parsing() {
        let json_str = r#"{
            "version": "5.0.0",
            "flags": {},
            "shapes": [
                {
                    "label": "person",
                    "points": [[100, 100], [200, 200]],
                    "group_id": null,
                    "shape_type": "polygon",
                    "flags": {}
                },
                {
                    "label": "car",
                    "points": [[300, 300], [400, 400]],
                    "group_id": null,
                    "shape_type": "rectangle",
                    "flags": {}
                }
            ],
            "imagePath": "test.jpg",
            "imageData": null,
            "imageHeight": 1080,
            "imageWidth": 1920
        }"#;

        let annotation: LabelMeAnnotation = serde_json::from_str(json_str)
            .expect("Failed to parse LabelMe JSON");

        assert_eq!(annotation.version, "5.0.0");
        assert_eq!(annotation.image_path, "test.jpg");
        assert_eq!(annotation.image_width, 1920);
        assert_eq!(annotation.image_height, 1080);
        assert_eq!(annotation.shapes.len(), 2);
        assert_eq!(annotation.shapes[0].label, "person");
        assert_eq!(annotation.shapes[1].label, "car");

        println!("✅ LabelMe JSON 解析測試通過!");
        println!("  找到標籤: {:?}", annotation.shapes.iter().map(|s| &s.label).collect::<Vec<_>>());
    }
}

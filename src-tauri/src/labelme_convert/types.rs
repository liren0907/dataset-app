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

/// Split data containers
#[derive(Debug, Default)]
pub struct SplitData {
    pub train_files: Vec<PathBuf>,
    pub val_files: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
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
    pub background_images: usize,
    pub labels_found: Vec<String>,
    pub skipped_labels: Vec<String>,
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

// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use chrono;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output format for dataset export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Yolo,
    Coco,
    /// LabelMe to LabelMe (filter/reorder labels, no train/val/test split)
    LabelMe,
}

/// Annotation format for YOLO export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationFormat {
    #[default]
    Bbox,
    Polygon,
}

/// Segmentation mode for COCO export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SegmentationMode {
    #[default]
    Polygon,
    BboxOnly,
}

/// Source for category definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CategoriesSource {
    /// Use provided label list
    LabelList(Vec<String>),
    /// Infer from JSON files
    Inferred,
    /// Read from external file
    File(PathBuf),
}

impl Default for CategoriesSource {
    fn default() -> Self {
        Self::Inferred
    }
}

/// Configuration for LabelMe to YOLO/COCO conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionConfig {
    /// Input directory containing LabelMe JSON files
    pub input_dir: PathBuf,

    /// Output directory for converted dataset (optional, defaults to input_dir)
    pub output_dir: Option<PathBuf>,

    /// Custom dataset folder name (optional, uses generated name if empty)
    #[serde(default)]
    pub custom_dataset_name: Option<String>,

    /// Output format (YOLO or COCO)
    #[serde(default)]
    pub output_format: OutputFormat,

    /// Annotation format for YOLO (bbox or polygon)
    #[serde(default)]
    pub annotation_format: AnnotationFormat,

    /// Proportion of dataset for validation (0.0 - 1.0)
    #[serde(default = "default_val_size")]
    pub val_size: f32,

    /// Proportion of dataset for testing (0.0 - 1.0)
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

    /// Number of worker threads (0 = auto)
    #[serde(default)]
    pub workers: usize,

    // COCO-specific options
    /// Segmentation mode for COCO export
    #[serde(default)]
    pub segmentation_mode: SegmentationMode,

    /// Starting image ID for COCO export
    #[serde(default = "default_start_id")]
    pub start_image_id: u32,

    /// Starting annotation ID for COCO export
    #[serde(default = "default_start_id")]
    pub start_annotation_id: u32,

    /// Categories source for COCO export
    #[serde(default)]
    pub categories_source: CategoriesSource,

    // LabelMe-specific options
    /// Skip train/val/test split (for LabelMe output)
    #[serde(default)]
    pub skip_split: bool,

    /// Remove imageData from output JSON (for LabelMe output)
    #[serde(default)]
    pub remove_image_data: bool,
}

fn default_val_size() -> f32 {
    0.2
}

fn default_seed() -> u64 {
    42
}

fn default_start_id() -> u32 {
    1
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            output_dir: None,
            custom_dataset_name: None,
            output_format: OutputFormat::default(),
            annotation_format: AnnotationFormat::default(),
            val_size: default_val_size(),
            test_size: 0.0,
            seed: default_seed(),
            include_background: false,
            label_list: Vec::new(),
            deterministic_labels: false,
            workers: 0,
            segmentation_mode: SegmentationMode::default(),
            start_image_id: default_start_id(),
            start_annotation_id: default_start_id(),
            categories_source: CategoriesSource::default(),
            // LabelMe-specific
            skip_split: false,
            remove_image_data: false,
        }
    }
}

impl ConversionConfig {
    /// Create a new config with input directory
    pub fn new(input_dir: PathBuf) -> Self {
        Self {
            input_dir,
            ..Default::default()
        }
    }

    /// Get the effective output directory
    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone().unwrap_or_else(|| self.input_dir.clone())
    }

    /// Generate the dataset folder name
    /// If custom_dataset_name is set, use it; otherwise generate from source folder + format + datetime
    pub fn get_dataset_folder_name(&self) -> String {
        if let Some(ref custom_name) = self.custom_dataset_name {
            if !custom_name.trim().is_empty() {
                return custom_name.trim().to_string();
            }
        }

        // Generate default name: {source_folder}_{format}_{annotation}_{datetime}
        let source_name = self.input_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("dataset");

        let format_str = match self.output_format {
            OutputFormat::Yolo => "yolo",
            OutputFormat::Coco => "coco",
            OutputFormat::LabelMe => "labelme",
        };

        let annotation_str = match self.annotation_format {
            AnnotationFormat::Bbox => "bbox",
            AnnotationFormat::Polygon => "polygon",
        };

        let datetime = chrono::Local::now().format("%Y%m%d_%H%M%S");

        format!("{}_{}_{}_{}", source_name, format_str, annotation_str, datetime)
    }

    /// Builder pattern: set custom dataset name
    pub fn with_custom_name(mut self, name: Option<String>) -> Self {
        self.custom_dataset_name = name;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.input_dir.exists() {
            return Err(format!(
                "Input directory does not exist: {}",
                self.input_dir.display()
            ));
        }

        if self.val_size < 0.0 || self.val_size > 1.0 {
            return Err(format!(
                "val_size must be between 0.0 and 1.0, got {}",
                self.val_size
            ));
        }

        if self.test_size < 0.0 || self.test_size > 1.0 {
            return Err(format!(
                "test_size must be between 0.0 and 1.0, got {}",
                self.test_size
            ));
        }

        if self.val_size + self.test_size > 1.0 {
            return Err(format!(
                "val_size + test_size must not exceed 1.0, got {}",
                self.val_size + self.test_size
            ));
        }

        Ok(())
    }

    /// Builder pattern: set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Builder pattern: set annotation format
    pub fn with_annotation_format(mut self, format: AnnotationFormat) -> Self {
        self.annotation_format = format;
        self
    }

    /// Builder pattern: set validation size
    pub fn with_val_size(mut self, size: f32) -> Self {
        self.val_size = size;
        self
    }

    /// Builder pattern: set test size
    pub fn with_test_size(mut self, size: f32) -> Self {
        self.test_size = size;
        self
    }

    /// Builder pattern: set label list
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.label_list = labels;
        self
    }

    /// Builder pattern: set output directory
    pub fn with_output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = Some(dir);
        self
    }

    /// Builder pattern: include background images
    pub fn with_background(mut self, include: bool) -> Self {
        self.include_background = include;
        self
    }

    /// Builder pattern: set seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Check if test split is enabled
    pub fn has_test_split(&self) -> bool {
        self.test_size > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ConversionConfig::default();
        assert_eq!(config.val_size, 0.2);
        assert_eq!(config.test_size, 0.0);
        assert_eq!(config.seed, 42);
        assert!(!config.include_background);
    }

    #[test]
    fn test_validation() {
        let mut config = ConversionConfig::default();
        config.input_dir = PathBuf::from(".");

        assert!(config.validate().is_ok());

        config.val_size = 1.5;
        assert!(config.validate().is_err());

        config.val_size = 0.5;
        config.test_size = 0.6;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_builder_pattern() {
        let config = ConversionConfig::new(PathBuf::from("."))
            .with_output_format(OutputFormat::Coco)
            .with_val_size(0.15)
            .with_test_size(0.15)
            .with_labels(vec!["cat".to_string(), "dog".to_string()]);

        assert_eq!(config.output_format, OutputFormat::Coco);
        assert_eq!(config.val_size, 0.15);
        assert_eq!(config.test_size, 0.15);
        assert_eq!(config.label_list.len(), 2);
    }
}

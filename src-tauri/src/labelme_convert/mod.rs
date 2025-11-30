// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

//! LabelMe format conversion module
//!
//! This module provides functionality to convert LabelMe JSON annotations to:
//! - YOLO format (for object detection and segmentation)
//! - COCO format (for instance segmentation and object detection)
//!
//! # Example
//!
//! ```rust,ignore
//! use labelme_convert::{ConversionConfig, OutputFormat, convert};
//! use std::path::PathBuf;
//!
//! let config = ConversionConfig::new(PathBuf::from("./data"))
//!     .with_output_format(OutputFormat::Yolo)
//!     .with_val_size(0.2)
//!     .with_test_size(0.1);
//!
//! let result = convert(&config);
//! ```

pub mod coco;
pub mod config;
pub mod conversion;
pub mod detection;
pub mod io;
pub mod labelme_out;
pub mod pipeline;
pub mod types;
pub mod yolo;

// Re-export commonly used types for convenience
pub use config::{
    AnnotationFormat, ConversionConfig, OutputFormat, SegmentationMode,
};
pub use detection::{analyze_dataset, validate_shape_points, DatasetAnalysis};
pub use types::{ConversionResult, InputAnnotationFormat};

/// Main conversion function that dispatches to the appropriate converter
/// based on the output format specified in the configuration.
///
/// # Arguments
///
/// * `config` - The conversion configuration
///
/// # Returns
///
/// A `ConversionResult` containing the output directory path, processing statistics,
/// and any errors that occurred during conversion.
///
/// # Example
///
/// ```rust,ignore
/// use labelme_convert::{ConversionConfig, OutputFormat, convert};
/// use std::path::PathBuf;
///
/// let config = ConversionConfig::new(PathBuf::from("./input"))
///     .with_output_format(OutputFormat::Yolo)
///     .with_val_size(0.2);
///
/// let result = convert(&config);
///
/// if result.success {
///     println!("Converted {} files to {}", result.stats.processed_files, result.output_dir);
/// } else {
///     for error in &result.errors {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
pub fn convert(config: &ConversionConfig) -> ConversionResult {
    // Auto-detect input format if not already set
    let mut config = config.clone();
    if config.detected_input_format.is_none() {
        let analysis = detection::analyze_dataset(&config.input_dir);
        println!(
            "📊 Auto-detected input format: {:?} (confidence: {:.1}%)",
            analysis.input_format,
            analysis.confidence * 100.0
        );
        println!("   {}", analysis.format_description);
        config.detected_input_format = Some(analysis.input_format);
    }

    match config.output_format {
        OutputFormat::Yolo => yolo::convert_to_yolo(&config),
        OutputFormat::Coco => coco::convert_to_coco(&config),
        OutputFormat::LabelMe => labelme_out::convert_to_labelme(&config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_module_reexports() {
        // Verify that commonly used types are accessible
        let _config = ConversionConfig::new(PathBuf::from("."));
        let _format = OutputFormat::Yolo;
        let _ann_format = AnnotationFormat::Bbox;
    }
}

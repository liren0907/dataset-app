//! Conversion Pipeline Abstraction
//!
//! This module defines the `ConversionPipeline` trait that provides a unified
//! interface for different output formats (YOLO, COCO, LabelMe).
//!
//! The pipeline abstraction allows:
//! - Different formats to have different split requirements (or no split)
//! - Unified file processing flow
//! - Easy addition of new output formats

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::types::{InvalidAnnotation, ProcessingStats};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Dataset split type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Split {
    Train,
    Val,
    Test,
    /// No split - all files go to a single output directory
    None,
}

impl Split {
    pub fn as_str(&self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::Val => "val",
            Split::Test => "test",
            Split::None => "",
        }
    }
}

/// File type for output path resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Image,
    Label,
    /// Annotation file (e.g., COCO JSON, LabelMe JSON)
    Annotation,
}

/// Result of processing a single file
#[derive(Debug, Default)]
pub struct ProcessedFileResult {
    /// Number of annotations successfully processed
    pub annotations_processed: usize,
    /// Number of annotations skipped
    pub annotations_skipped: usize,
    /// List of invalid annotations with reasons
    pub invalid_annotations: Vec<InvalidAnnotation>,
    /// Whether this image became empty after label filtering
    /// (had annotations originally but all were filtered out)
    pub is_filtered_empty: bool,
    /// File name for tracking (only set when is_filtered_empty is true)
    pub filtered_empty_file_name: Option<String>,
}

/// Shared context during conversion process
#[derive(Debug)]
pub struct ProcessingContext {
    /// Processing statistics
    pub stats: ProcessingStats,
    /// Label name to ID mapping
    pub label_map: HashMap<String, usize>,
    /// Set of processed image paths (to avoid duplicates)
    pub processed_images: HashSet<String>,
    /// Labels that were skipped (not in the allowed list)
    pub skipped_labels: HashSet<String>,
    /// Accumulated errors
    pub errors: Vec<String>,
}

impl ProcessingContext {
    /// Create a new processing context
    pub fn new() -> Self {
        Self {
            stats: ProcessingStats::new(),
            label_map: HashMap::new(),
            processed_images: HashSet::new(),
            skipped_labels: HashSet::new(),
            errors: Vec::new(),
        }
    }

    /// Initialize context with predefined labels
    pub fn with_labels(labels: &[String]) -> Self {
        let mut ctx = Self::new();
        for (id, label) in labels.iter().enumerate() {
            ctx.label_map.insert(label.clone(), id);
        }
        ctx
    }

    /// Add a label to the map if not already present
    pub fn ensure_label(&mut self, label: &str) -> usize {
        if let Some(&id) = self.label_map.get(label) {
            id
        } else {
            let next_id = self.label_map.len();
            self.label_map.insert(label.to_string(), next_id);
            next_id
        }
    }

    /// Record an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Record a skipped label
    pub fn add_skipped_label(&mut self, label: &str) {
        self.skipped_labels.insert(label.to_string());
    }

    /// Check if an image has already been processed
    pub fn is_image_processed(&self, image_key: &str) -> bool {
        self.processed_images.contains(image_key)
    }

    /// Mark an image as processed
    pub fn mark_image_processed(&mut self, image_key: String) {
        self.processed_images.insert(image_key);
    }
}

impl Default for ProcessingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstract interface for output directories
///
/// Different output formats have different directory structures:
/// - YOLO: images/train, images/val, labels/train, labels/val
/// - COCO: images/train, images/val, annotations/
/// - LabelMe: single output directory with JSON and images
pub trait OutputDirectories: Send + Sync {
    /// Get the base output directory
    fn base_dir(&self) -> &Path;

    /// Get the output path for a specific file type and split
    ///
    /// # Arguments
    /// * `split` - The dataset split (Train/Val/Test/None)
    /// * `file_type` - The type of file (Image/Label/Annotation)
    ///
    /// # Returns
    /// The directory path for the specified file type and split
    fn get_output_dir(&self, split: Split, file_type: FileType) -> &Path;

    /// Check if this output format uses train/val/test splits
    fn uses_splits(&self) -> bool;
}

/// The main conversion pipeline trait
///
/// Implementors define how files are processed and output for each format.
pub trait ConversionPipeline: Send + Sync {
    /// Whether this format requires train/val/test splitting
    fn needs_split(&self) -> bool;

    /// Set up output directories for this format
    ///
    /// # Arguments
    /// * `config` - The conversion configuration
    ///
    /// # Returns
    /// A boxed OutputDirectories implementation
    fn setup_output_dirs(
        &self,
        config: &ConversionConfig,
    ) -> Result<Box<dyn OutputDirectories>, String>;

    /// Gather all labels from JSON files (first pass for deterministic labeling)
    ///
    /// Default implementation scans all JSON files and collects unique labels.
    fn gather_labels(
        &self,
        json_files: &[PathBuf],
        context: &mut ProcessingContext,
    ) {
        use crate::labelme_convert::io::read_labelme_json;

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
            context.label_map.insert(label, id);
        }
    }

    /// Process a single LabelMe JSON file
    ///
    /// # Arguments
    /// * `json_path` - Path to the JSON file
    /// * `config` - Conversion configuration
    /// * `output_dirs` - Output directory structure
    /// * `context` - Shared processing context
    ///
    /// # Returns
    /// Result containing processing statistics for this file
    fn process_file(
        &self,
        json_path: &Path,
        config: &ConversionConfig,
        output_dirs: &dyn OutputDirectories,
        context: &mut ProcessingContext,
    ) -> Result<ProcessedFileResult, String>;

    /// Finalize the conversion (e.g., write dataset.yaml, COCO JSON)
    ///
    /// # Arguments
    /// * `config` - Conversion configuration
    /// * `output_dirs` - Output directory structure
    /// * `context` - Shared processing context
    fn finalize(
        &self,
        config: &ConversionConfig,
        output_dirs: &dyn OutputDirectories,
        context: &ProcessingContext,
    ) -> Result<(), String>;
}

/// Determine which split a file belongs to based on its path hash
///
/// This provides deterministic splitting based on file path.
///
/// # Arguments
/// * `path_hash` - Hash of the file path
/// * `val_size` - Proportion for validation set (0.0 - 1.0)
/// * `test_size` - Proportion for test set (0.0 - 1.0)
///
/// # Returns
/// The split this file should belong to
pub fn determine_split(path_hash: u64, val_size: f32, test_size: f32) -> Split {
    let ratio = (path_hash % 1000) as f32 / 1000.0;

    if ratio < val_size {
        Split::Val
    } else if ratio < val_size + test_size {
        Split::Test
    } else {
        Split::Train
    }
}

/// Calculate hash for a string (for deterministic splitting)
pub fn hash_string(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_context_new() {
        let ctx = ProcessingContext::new();
        assert!(ctx.label_map.is_empty());
        assert!(ctx.processed_images.is_empty());
        assert!(ctx.errors.is_empty());
    }

    #[test]
    fn test_processing_context_with_labels() {
        let labels = vec!["cat".to_string(), "dog".to_string()];
        let ctx = ProcessingContext::with_labels(&labels);
        assert_eq!(ctx.label_map.get("cat"), Some(&0));
        assert_eq!(ctx.label_map.get("dog"), Some(&1));
    }

    #[test]
    fn test_ensure_label() {
        let mut ctx = ProcessingContext::new();
        let id1 = ctx.ensure_label("cat");
        let id2 = ctx.ensure_label("dog");
        let id3 = ctx.ensure_label("cat"); // Should return same ID

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // Same as first
    }

    #[test]
    fn test_determine_split() {
        // Test with specific hash values
        assert_eq!(determine_split(100, 0.2, 0.1), Split::Val); // 0.1 < 0.2
        assert_eq!(determine_split(250, 0.2, 0.1), Split::Test); // 0.25 between 0.2 and 0.3
        assert_eq!(determine_split(500, 0.2, 0.1), Split::Train); // 0.5 > 0.3
    }

    #[test]
    fn test_split_as_str() {
        assert_eq!(Split::Train.as_str(), "train");
        assert_eq!(Split::Val.as_str(), "val");
        assert_eq!(Split::Test.as_str(), "test");
        assert_eq!(Split::None.as_str(), "");
    }
}

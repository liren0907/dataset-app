/// Scanner module for analyzing LabelMe datasets
///
/// This module provides async, parallelized scanning capabilities for:
/// - Label detection
/// - Label counting
/// - Format analysis

pub mod label_scanner;
pub mod count_scanner;
pub mod format_analyzer;

// Re-export main functions for convenience
pub use label_scanner::scan_labels_async;
pub use count_scanner::scan_labels_with_counts_async;
pub use format_analyzer::analyze_dataset_async;

/// Async format analyzer with progress reporting
///
/// This module wraps the existing detection::analyze_dataset function
/// in an async interface with progress reporting.
use crate::labelme_convert::{
    detection::{analyze_dataset_with_config, AnalysisConfig, DatasetAnalysis},
    progress::ProgressEmitter,
};
use std::path::PathBuf;

/// Asynchronously analyze a LabelMe dataset to detect annotation format
///
/// This function wraps the existing blocking `analyze_dataset` function
/// and adds progress reporting and async execution.
///
/// # Arguments
/// * `input_dir` - Path to the directory containing LabelMe JSON files
/// * `progress` - Optional progress emitter for UI updates
///
/// # Returns
/// A `DatasetAnalysis` containing format information and confidence scores
pub async fn analyze_dataset_async(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<DatasetAnalysis, String> {
    // Move to blocking thread pool to avoid blocking the async runtime
    let result = tokio::task::spawn_blocking(move || analyze_dataset_blocking(input_dir, progress))
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// Blocking implementation of dataset analysis
fn analyze_dataset_blocking(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<DatasetAnalysis, String> {
    if !input_dir.exists() {
        return Err(format!("Directory does not exist: {:?}", input_dir));
    }

    if let Some(ref p) = progress {
        p.emit(0, 100, "開始分析資料集格式...");
    }

    // Use default config for analysis
    let config = AnalysisConfig::default();

    if let Some(ref p) = progress {
        p.emit(30, 100, "讀取樣本檔案...");
    }

    // Call the existing synchronous analysis function
    let analysis = analyze_dataset_with_config(&input_dir, &config);

    if let Some(ref p) = progress {
        p.emit(80, 100, "計算信心分數...");
    }

    if let Some(ref p) = progress {
        let confidence_percent = format!("{:.1}%", analysis.confidence * 100.0);
        p.complete(format!(
            "格式分析完成：{} (信心度：{})",
            analysis.format_description, confidence_percent
        ));
    }

    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_nonexistent_directory() {
        let result = analyze_dataset_async(PathBuf::from("/nonexistent/path"), None).await;
        assert!(result.is_err());
    }
}

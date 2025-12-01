/// Async label counter with progress reporting
///
/// This module counts occurrences of each label across all LabelMe JSON files.
/// It uses Rayon for parallel file processing.
use crate::labelme_convert::{io::find_json_files, progress::ProgressEmitter};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Asynchronously scan labels with occurrence counts
///
/// # Arguments
/// * `input_dir` - Path to the directory containing LabelMe JSON files
/// * `progress` - Optional progress emitter for UI updates
///
/// # Returns
/// A HashMap mapping label names to their occurrence counts
pub async fn scan_labels_with_counts_async(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<HashMap<String, usize>, String> {
    let result = tokio::task::spawn_blocking(move || scan_counts_blocking(input_dir, progress))
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// Blocking implementation of label counting (uses Rayon)
fn scan_counts_blocking(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<HashMap<String, usize>, String> {
    if !input_dir.exists() {
        return Err(format!("Directory does not exist: {:?}", input_dir));
    }

    let json_files = find_json_files(&input_dir);
    let total = json_files.len();

    if total == 0 {
        if let Some(ref p) = progress {
            p.complete("未找到 JSON 檔案");
        }
        return Ok(HashMap::new());
    }

    if let Some(ref p) = progress {
        p.emit(0, total, "開始統計標籤數量...");
    }

    // Thread-safe container for counts
    let label_counts = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    let processed = Arc::new(Mutex::new(0usize));

    // 🚀 Parallel processing with Rayon
    json_files.par_iter().for_each(|json_path| {
        if let Ok(content) = fs::read_to_string(json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(shapes) = json.get("shapes").and_then(|s| s.as_array()) {
                    // Collect local counts
                    let mut local_counts = HashMap::new();
                    for shape in shapes {
                        if let Some(label) = shape.get("label").and_then(|l| l.as_str()) {
                            *local_counts.entry(label.to_string()).or_insert(0) += 1;
                        }
                    }

                    // Merge into global counts
                    if !local_counts.is_empty() {
                        if let Ok(mut global_counts) = label_counts.lock() {
                            for (label, count) in local_counts {
                                *global_counts.entry(label).or_insert(0) += count;
                            }
                        }
                    }
                }
            }
        }

        // Update progress
        if let Some(ref p) = progress {
            if let Ok(mut count) = processed.lock() {
                *count += 1;
                if *count % 100 == 0 || *count == total {
                    p.emit(
                        *count,
                        total,
                        format!("已統計 {} / {} 個檔案", count, total),
                    );
                }
            }
        }
    });

    // Extract results
    let counts = Arc::try_unwrap(label_counts)
        .map_err(|_| "Failed to unwrap counts Arc")?
        .into_inner()
        .map_err(|e| format!("Mutex poisoned: {}", e))?;

    if let Some(ref p) = progress {
        let total_annotations: usize = counts.values().sum();
        p.complete(format!(
            "統計完成，共 {} 個標籤，{} 個標註",
            counts.len(),
            total_annotations
        ));
    }

    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_counts_empty_directory() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let result = scan_labels_with_counts_async(temp_dir.path().to_path_buf(), None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}

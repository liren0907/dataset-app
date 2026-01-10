/// Async label scanner with progress reporting
///
/// This module scans a LabelMe dataset directory and extracts all unique labels.
/// It uses Rayon for parallel file processing to maximize performance.

use crate::labelme_convert::{io::find_json_files, progress::ProgressEmitter};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Asynchronously scan all labels from a LabelMe dataset
///
/// This function:
/// 1. Finds all JSON files in the directory
/// 2. Spawns a blocking task to avoid blocking the async runtime
/// 3. Uses Rayon to process files in parallel
/// 4. Reports progress periodically
///
/// # Arguments
/// * `input_dir` - Path to the directory containing LabelMe JSON files
/// * `progress` - Optional progress emitter for UI updates
///
/// # Returns
/// A sorted list of unique label names
pub async fn scan_labels_async(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<Vec<String>, String> {
    // Move the heavy work to a blocking thread pool to avoid blocking the async runtime
    let result = tokio::task::spawn_blocking(move || {
        scan_labels_blocking(input_dir, progress)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// Blocking implementation of label scanning (uses Rayon for parallelization)
fn scan_labels_blocking(
    input_dir: PathBuf,
    progress: Option<ProgressEmitter>,
) -> Result<Vec<String>, String> {
    if !input_dir.exists() {
        return Err(format!("Directory does not exist: {:?}", input_dir));
    }

    let json_files = find_json_files(&input_dir);
    let total = json_files.len();

    if total == 0 {
        if let Some(ref p) = progress {
            p.complete("未找到 JSON 檔案");
        }
        return Ok(Vec::new());
    }

    if let Some(ref p) = progress {
        p.emit(0, total, "開始掃描標籤...");
    }

    // Thread-safe containers for collecting results
    let labels = Arc::new(Mutex::new(HashSet::new()));
    let processed = Arc::new(Mutex::new(0usize));

    // 🚀 Parallel processing with Rayon
    json_files.par_iter().for_each(|json_path| {
        // Fast path: read and parse JSON manually to extract labels
        if let Ok(content) = fs::read_to_string(json_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(shapes) = json.get("shapes").and_then(|s| s.as_array()) {
                    let mut local_labels = Vec::new();
                    for shape in shapes {
                        if let Some(label) = shape.get("label").and_then(|l| l.as_str()) {
                            local_labels.push(label.to_string());
                        }
                    }

                    // Merge into global set
                    if !local_labels.is_empty() {
                        if let Ok(mut global_labels) = labels.lock() {
                            global_labels.extend(local_labels);
                        }
                    }
                }
            }
        }

        // Update progress (report every 100 files or at completion)
        if let Some(ref p) = progress {
            if let Ok(mut count) = processed.lock() {
                *count += 1;
                if *count % 100 == 0 || *count == total {
                    p.emit(*count, total, format!("已掃描 {} / {} 個檔案", count, total));
                }
            }
        }
    });

    // Extract and sort results
    let labels = Arc::try_unwrap(labels)
        .map_err(|_| "Failed to unwrap labels Arc")?
        .into_inner()
        .map_err(|e| format!("Mutex poisoned: {}", e))?;

    let mut sorted_labels: Vec<_> = labels.into_iter().collect();
    sorted_labels.sort();

    if let Some(ref p) = progress {
        p.complete(format!(
            "掃描完成，找到 {} 個標籤",
            sorted_labels.len()
        ));
    }

    Ok(sorted_labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_empty_directory() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let result = scan_labels_async(temp_dir.path().to_path_buf(), None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}

// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::config::ConversionConfig;
use crate::labelme_convert::types::{
    is_image_extension, CocoOutputDirs, LabelMeAnnotation, YoloOutputDirs,
};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Set up output directories for YOLO dataset
pub fn setup_yolo_directories(config: &ConversionConfig) -> std::io::Result<YoloOutputDirs> {
    let dataset_name = config.get_dataset_folder_name();
    let base_dir = config.get_output_dir().join(&dataset_name);

    let labels_dir = base_dir.join("labels");
    let images_dir = base_dir.join("images");

    let train_labels_dir = labels_dir.join("train");
    let val_labels_dir = labels_dir.join("val");
    let train_images_dir = images_dir.join("train");
    let val_images_dir = images_dir.join("val");

    // Create directories
    fs::create_dir_all(&train_labels_dir)?;
    fs::create_dir_all(&val_labels_dir)?;
    fs::create_dir_all(&train_images_dir)?;
    fs::create_dir_all(&val_images_dir)?;

    let (test_labels_dir, test_images_dir) = if config.has_test_split() {
        let test_labels = labels_dir.join("test");
        let test_images = images_dir.join("test");
        fs::create_dir_all(&test_labels)?;
        fs::create_dir_all(&test_images)?;
        (Some(test_labels), Some(test_images))
    } else {
        (None, None)
    };

    Ok(YoloOutputDirs {
        base_dir,
        train_labels_dir,
        val_labels_dir,
        train_images_dir,
        val_images_dir,
        test_labels_dir,
        test_images_dir,
    })
}

/// Set up output directories for COCO dataset
pub fn setup_coco_directories(config: &ConversionConfig) -> std::io::Result<CocoOutputDirs> {
    let dataset_name = config.get_dataset_folder_name();
    let base_dir = config.get_output_dir().join(&dataset_name);

    let annotations_dir = base_dir.join("annotations");
    let images_dir = base_dir.join("images");

    let train_images_dir = images_dir.join("train");
    let val_images_dir = images_dir.join("val");

    // Create directories
    fs::create_dir_all(&annotations_dir)?;
    fs::create_dir_all(&train_images_dir)?;
    fs::create_dir_all(&val_images_dir)?;

    let test_images_dir = if config.has_test_split() {
        let test_images = images_dir.join("test");
        fs::create_dir_all(&test_images)?;
        Some(test_images)
    } else {
        None
    };

    Ok(CocoOutputDirs {
        base_dir,
        annotations_dir,
        train_images_dir,
        val_images_dir,
        test_images_dir,
    })
}

/// Read and parse a LabelMe JSON file
pub fn read_labelme_json(path: &Path) -> Result<LabelMeAnnotation, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;

    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Find all JSON files in a directory (recursively)
pub fn find_json_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "json")
                && !is_output_directory(e.path())
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Find all image files in a directory (recursively)
pub fn find_image_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |ext| is_image_extension(&ext.to_string_lossy()))
                && !is_output_directory(e.path())
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Check if a path is inside an output directory (to avoid processing converted files)
fn is_output_directory(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("YOLODataset") || path_str.contains("COCODataset")
}

/// Copy an image file to the destination directory
pub fn copy_image(src: &Path, dest_dir: &Path) -> std::io::Result<PathBuf> {
    let file_name = src.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid source path")
    })?;

    let dest_path = dest_dir.join(file_name);

    // Handle filename collision by adding a suffix
    let final_path = if dest_path.exists() {
        generate_unique_path(&dest_path)
    } else {
        dest_path
    };

    fs::copy(src, &final_path)?;
    Ok(final_path)
}

/// Generate a unique file path by adding a numeric suffix
fn generate_unique_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, ext)
        };
        let new_path = parent.join(&new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

/// Write content to a file
pub fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    Ok(())
}

/// Extract image data from base64 and save to file
pub fn extract_embedded_image(
    image_data: &str,
    dest_path: &Path,
) -> Result<(), String> {
    use base64::Engine;

    // Remove data URL prefix if present
    let base64_data = if let Some(pos) = image_data.find(",") {
        &image_data[pos + 1..]
    } else {
        image_data
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    fs::write(dest_path, decoded).map_err(|e| format!("Failed to write image: {}", e))?;

    Ok(())
}

/// Get the image path relative to the JSON file
pub fn resolve_image_path(json_path: &Path, image_path_str: &str) -> PathBuf {
    let image_path = PathBuf::from(image_path_str);

    if image_path.is_absolute() {
        image_path
    } else {
        json_path
            .parent()
            .map(|p| p.join(&image_path))
            .unwrap_or(image_path)
    }
}

/// Sanitize filename to avoid path issues
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Create dataset.yaml file for YOLO
pub fn create_dataset_yaml(
    output_dir: &Path,
    label_map: &std::collections::HashMap<String, usize>,
    has_test: bool,
) -> std::io::Result<()> {
    let yaml_path = output_dir.join("dataset.yaml");

    // Sort labels by ID
    let mut sorted_labels: Vec<_> = label_map.iter().collect();
    sorted_labels.sort_by_key(|(_, id)| *id);

    let mut content = String::new();

    // Use absolute path
    let abs_path = fs::canonicalize(output_dir)
        .unwrap_or_else(|_| output_dir.to_path_buf());

    content.push_str(&format!("path: {}\n", abs_path.display()));
    content.push_str("train: images/train\n");
    content.push_str("val: images/val\n");

    if has_test {
        content.push_str("test: images/test\n");
    } else {
        content.push_str("test:\n");
    }

    content.push_str("\nnames:\n");

    for (label, id) in sorted_labels {
        content.push_str(&format!("  {}: {}\n", id, label));
    }

    write_file(&yaml_path, &content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_output_directory() {
        assert!(is_output_directory(Path::new("/some/path/YOLODataset/images/train")));
        assert!(is_output_directory(Path::new("/some/path/COCODataset/annotations")));
        assert!(!is_output_directory(Path::new("/some/path/data/images")));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test/file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("test:file.txt"), "test_file.txt");
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
    }

    #[test]
    fn test_resolve_image_path() {
        let json_path = Path::new("/data/annotations/image1.json");

        // Relative path
        let resolved = resolve_image_path(json_path, "../images/image1.jpg");
        assert!(resolved.to_string_lossy().contains("images"));

        // Absolute path
        let resolved = resolve_image_path(json_path, "/absolute/path/image.jpg");
        assert_eq!(resolved, PathBuf::from("/absolute/path/image.jpg"));
    }
}

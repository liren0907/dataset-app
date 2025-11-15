use crate::core::labelme2yolo::YoloExporter;
use crate::core::yolo_to_labelme_converter::YoloToLabelmeConverter;
use std::fs;

#[tauri::command]
pub fn export_to_yolo(
    source_dir: String,
    output_dir: String,
    train_ratio: f32,
    val_ratio: f32,
    test_ratio: f32,
    shape: String,
    specific_labels: bool,
    class_names_str: String,
) -> Result<String, String> {
    // Parse class names from comma-separated string
    let class_names = if specific_labels && !class_names_str.is_empty() {
        class_names_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create exporter
    let exporter = YoloExporter::new(
        &source_dir,
        &output_dir,
        train_ratio,
        val_ratio,
        test_ratio,
        &shape,
        specific_labels,
        class_names,
    );

    // Export
    match exporter.export() {
        Ok(_) => Ok("Dataset exported to YOLO format successfully".to_string()),
        Err(e) => Err(format!("Failed to export to YOLO format: {}", e)),
    }
}

#[tauri::command]
pub fn convert_to_labelme(
    source_dir: String,
    output_dir: String,
    version: String,
    class_label_file: String,
) -> Result<String, String> {
    // Create output directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&output_dir) {
        return Err(format!("Failed to create output directory: {}", e));
    }

    // Create converter
    let mut converter =
        YoloToLabelmeConverter::new(&source_dir, &output_dir, &version, &class_label_file);

    // Convert
    match converter.convert() {
        Ok(_) => Ok("Dataset converted to LabelMe format successfully".to_string()),
        Err(e) => Err(format!("Failed to convert to LabelMe format: {}", e)),
    }
}
# Tauri Commands Documentation

## Command Signatures Reference

> **Note**: The application now exposes **26 Tauri commands** (updated from original 18), including new LabelMe conversion commands.

### Media Commands (3)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `get_video_info` | `filename: &str` | `Result<String, String>` |
| `read_video_file` | `file_path: String` | `Result<String, String>` |
| `read_image_file` | `file_path: String` | `Result<String, String>` |

---

### File Operations (2)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `read_file_content` | `file_path: String` | `Result<String, String>` |
| `open_file_dialog` | (none) | `Result<String, String>` |

---

### Drawing Commands (3)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `draw_bounding_boxes` | `source_dir: String, json_path: String, output_dir: String` | `Result<String, String>` |
| `draw_polygons` | `source_dir: String, json_path: String, output_dir: String` | `Result<String, String>` |
| `visualize_dataset` | `source_dir: String, output_dir: String, annotation_type: String, save_output: bool` | `Result<String, String>` |

---

### Export Commands (2)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `export_to_yolo` | `source_dir: String, output_dir: String, train_ratio: f32, val_ratio: f32, test_ratio: f32, shape: String, specific_labels: bool, class_names_str: String` | `Result<String, String>` |
| `convert_to_labelme` | `source_dir: String, output_dir: String, version: String, class_label_file: String` | `Result<String, String>` |

---

### Label Management (3)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `change_label_name` | `source_dir: String, output_dir: String, old_label: String, new_label: String` | `Result<String, String>` |
| `clear_image_data` | `source_dir: String, output_dir: String` | `Result<String, String>` |
| `extract_labels` | `source_dir: String, output_dir: String, labels_str: String` | `Result<String, String>` |

---

### Dataset Analysis (2)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `get_dataset_stats` | `source_dir: String` | `Result<String, String>` |
| `get_labelme_summary` | `path: &str` | `Result<String, String>` |

---

### Directory Operations (4)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `get_directory_images` | `path: String` | `Result<String, String>` |
| `get_paginated_images` | `path: String, page: usize, page_size: usize` | `Result<String, String>` |
| `get_image_details` | `path: String` | `Result<String, String>` |
| `auto_annotate_images` | `path: String, page: usize, page_size: usize` | `Result<String, String>` |

---

### Advanced Processing (3)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `crop_and_remap_annotations` | `source_dir: String, output_dir: String, parent_label: String, required_child_labels_str: String, padding_factor: f32` | `Result<String, String>` |
| `generate_annotated_previews` | `source_dir: String, num_previews: usize, temp_dir: String` | `Result<String, String>` |
| `crop_remap_adapter` | `source_dir: String, num_previews: usize` | `Result<String, String>` |

---

### LabelMe Conversion Commands (10)

#### Synchronous Commands

| Command | Parameters | Returns |
|---------|-----------|---------|
| `convert_labelme` | `request: ConvertLabelMeRequest` | `Result<ConversionResult, String>` |
| `quick_convert_to_yolo` | `input_dir: String, val_size: Option<f32>, use_polygon: Option<bool>` | `Result<ConversionResult, String>` |
| `quick_convert_to_coco` | `input_dir: String, val_size: Option<f32>` | `Result<ConversionResult, String>` |
| `scan_labelme_labels` | `input_dir: String` | `Result<Vec<String>, String>` |
| `scan_labelme_labels_with_counts` | `input_dir: String` | `Result<HashMap<String, usize>, String>` |
| `count_labelme_files` | `input_dir: String` | `Result<usize, String>` |
| `analyze_labelme_dataset` | `input_dir: String` | `Result<DatasetAnalysisResponse, String>` |

#### Async Commands (with Progress Reporting)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `scan_labelme_labels_async` | `window: tauri::Window, input_dir: String` | `Result<Vec<String>, String>` |
| `scan_labelme_labels_with_counts_async` | `window: tauri::Window, input_dir: String` | `Result<HashMap<String, usize>, String>` |
| `analyze_labelme_dataset_async` | `window: tauri::Window, input_dir: String` | `Result<DatasetAnalysisResponse, String>` |

---

### External Module Functions (2)

| Command | Parameters | Returns |
|---------|-----------|---------|
| `export_to_yolo_new` | `source_dir: String, output_dir: String, train_ratio: f32, val_ratio: f32, test_ratio: f32, shape: String, specific_labels: bool, class_names_str: String` | `Result<String, String>` |
| `generate_single_annotated_preview` | `image_path: String` | `Result<String, String>` |

---


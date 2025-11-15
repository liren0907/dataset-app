# Tauri Commands Documentation

This comprehensive documentation serves as the definitive reference for the Dataset App's Tauri backend architecture, providing a complete hierarchical breakdown of all 18 Tauri commands and their underlying Rust implementation. The documentation is structured as an intuitive tree visualization that maps the exact modular organization found in the source code, showing both the public API layer (commands) and the implementation layer (core modules). Each module entry details its public functions and data structures, enabling developers to understand the complete API surface and internal architecture. The hierarchical structure directly reflects the Rust module system, with clear separation between command handlers (API endpoints) and core business logic modules, providing both high-level architectural overview and detailed implementation reference for maintenance, debugging, and feature development.

## Command Overview

The application exposes 18 Tauri commands, organized into logical categories:

### Media Commands (3)
- `get_video_info` - Get video file information
- `read_video_file` - Read video file content
- `read_image_file` - Read image file information

### File Operations (2)
- `read_file_content` - Read text file content
- `open_file_dialog` - Open file selection dialog

### Drawing Commands (3)
- `draw_bounding_boxes` - Draw bounding boxes on images
- `draw_polygons` - Draw polygons on images
- `visualize_dataset` - Visualize entire dataset annotations

### Export Commands (2)
- `export_to_yolo` - Export LabelMe annotations to YOLO format
- `convert_to_labelme` - Convert YOLO to LabelMe format

### Label Management (3)
- `change_label_name` - Change annotation label names
- `clear_image_data` - Clear image data from annotations
- `extract_labels` - Extract specific labels from dataset

### Dataset Analysis (2)
- `get_dataset_stats` - Get comprehensive dataset statistics
- `get_labelme_summary` - Get LabelMe dataset summary

### Directory Operations (4)
- `get_directory_images` - Get all images in directory
- `get_paginated_images` - Get paginated image list
- `get_image_details` - Get detailed image information
- `auto_annotate_images` - Auto-annotate images with existing data

### Advanced Processing (3)
- `crop_and_remap_annotations` - Crop and remap annotations around parent labels
- `generate_annotated_previews` - Generate preview images with annotations
- `crop_remap_adapter` - Adapter for crop/remap functionality

---

## Rust Backend Structure

```
src-tauri/src/
├── main.rs
│   ├── Functions:
│   │   └── main
│   └── Structs:
│       └── (none)
├── commands/
│   ├── mod.rs
│   │   ├── Functions:
│   │   │   └── (none - module declarations only)
│   │   └── Structs:
│   │       └── (none)
│   ├── advanced.rs
│   │   ├── Functions:
│   │   │   ├── crop_and_remap_annotations
│   │   │   ├── generate_annotated_previews
│   │   │   └── crop_remap_adapter
│   │   └── Structs:
│   │       └── (none)
│   ├── dataset.rs
│   │   ├── Functions:
│   │   │   ├── get_dataset_stats
│   │   │   └── get_labelme_summary
│   │   └── Structs:
│   │       └── (none)
│   ├── directory.rs
│   │   ├── Functions:
│   │   │   ├── get_directory_images
│   │   │   ├── get_paginated_images
│   │   │   ├── get_image_details
│   │   │   └── auto_annotate_images
│   │   └── Structs:
│   │       └── (none)
│   ├── drawing.rs
│   │   ├── Functions:
│   │   │   ├── draw_bounding_boxes
│   │   │   ├── draw_polygons
│   │   │   └── visualize_dataset
│   │   └── Structs:
│   │       └── (none)
│   ├── export.rs
│   │   ├── Functions:
│   │   │   ├── export_to_yolo
│   │   │   └── convert_to_labelme
│   │   └── Structs:
│   │       └── (none)
│   ├── file_ops.rs
│   │   ├── Functions:
│   │   │   ├── read_file_content
│   │   │   └── open_file_dialog
│   │   └── Structs:
│   │       └── (none)
│   ├── labels.rs
│   │   ├── Functions:
│   │   │   ├── change_label_name
│   │   │   ├── clear_image_data
│   │   │   └── extract_labels
│   │   └── Structs:
│   │       └── (none)
│   └── media.rs
│       ├── Functions:
│       │   ├── get_video_info
│       │   ├── read_video_file
│       │   └── read_image_file
│       └── Structs:
│           └── (none)
├── core/
│   ├── mod.rs
│   │   ├── Functions:
│   │   │   └── (none - module declarations only)
│   │   └── Structs:
│   │       └── (none)
│   ├── annotation_processor.rs
│   │   ├── Functions:
│   │   │   └── process_parent_child_annotations
│   │   └── Structs:
│   │       └── (none)
│   ├── bounding_box_drawer.rs
│   │   ├── Functions:
│   │   │   └── draw_bounding_boxes
│   │   └── Structs:
│   │       └── (none)
│   ├── data_visualizer.rs
│   │   ├── Functions:
│   │   │   └── visualize_dataset
│   │   └── Structs:
│   │       └── (none)
│   ├── dialog_handler.rs
│   │   ├── Functions:
│   │   │   └── select_video_file
│   │   └── Structs:
│   │       └── (none)
│   ├── directory_handler.rs
│   │   ├── Functions:
│   │   │   ├── get_directory_images
│   │   │   ├── get_paginated_images
│   │   │   └── get_image_details
│   │   └── Structs:
│   │       ├── ImageFile
│   │       ├── ImageDimensions
│   │       └── PaginatedResult
│   ├── file_operations.rs
│   │   ├── Functions:
│   │   │   └── process_directories
│   │   └── Structs:
│   │       └── (none)
│   ├── image_annotator.rs
│   │   ├── Functions:
│   │   │   └── auto_annotate_images
│   │   └── Structs:
│   │       ├── Annotation
│   │       ├── AnnotatedImage
│   │       └── AnnotationResult
│   ├── image_handler.rs
│   │   ├── Functions:
│   │   │   └── read_image_info
│   │   └── Structs:
│   │       └── (none)
│   ├── labelme_types.rs
│   │   ├── Functions:
│   │   │   └── get_bounding_box
│   │   └── Structs:
│   │       ├── LabelMeShape
│   │       ├── LabelMeFile
│   │       └── BoundingBox
│   ├── labelme_viewer.rs
│   │   ├── Functions:
│   │   │   └── get_labelme_summary
│   │   └── Structs:
│   │       └── LabelmeViewerModule
│   ├── labelme2yolo.rs
│   │   ├── Functions:
│   │   │   ├── export_to_yolo
│   │   │   └── export_to_yolo_new
│   │   └── Structs:
│   │       └── YoloExporter
│   ├── polygon_drawer.rs
│   │   ├── Functions:
│   │   │   └── draw_polygons
│   │   └── Structs:
│   │       └── (none)
│   ├── preview.rs
│   │   ├── Functions:
│   │   │   ├── generate_single_annotated_preview
│   │   │   └── generate_annotated_previews
│   │   └── Structs:
│   │       └── (none)
│   ├── video_handler.rs
│   │   ├── Functions:
│   │   │   ├── get_video_info
│   │   │   └── read_video_file
│   │   └── Structs:
│   │       └── (none)
│   ├── video_tools.rs
│   │   ├── Functions:
│   │   │   ├── process_single_video
│   │   │   └── extract_frames_from_videos
│   │   └── Structs:
│   │       ├── OutputMode
│   │       ├── FileFormat
│   │       ├── ExtractionConfig
│   │       └── Config
│   └── yolo_to_labelme_converter.rs
│       ├── Functions:
│       │   └── (none)
│       └── Structs:
│           └── YoloToLabelmeConverter
└── crop_remap/
    └── mod.rs
        ├── Functions:
        │   └── crop_remap_adapter
        └── Structs:
            └── (none)
```

---

## Additional Notes

- All commands return `Result<String, String>` where the `String` contains either JSON data or success/error messages
- Commands are organized into logical modules for maintainability
- Error handling is consistent across all commands
- Input validation is performed at the command level
- Most commands create output directories automatically if they don't exist

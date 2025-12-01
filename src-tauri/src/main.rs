// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;
mod crop_remap;
mod labelme_convert;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Media commands
            media::get_video_info,
            media::read_video_file,
            media::read_image_file,
            // File operations
            file_ops::read_file_content,
            file_ops::open_file_dialog,
            // Drawing commands
            drawing::draw_bounding_boxes,
            drawing::draw_polygons,
            drawing::visualize_dataset,
            // Export commands
            export::export_to_yolo,
            export::convert_to_labelme,
            // Label management
            labels::change_label_name,
            labels::clear_image_data,
            labels::extract_labels,
            // Dataset analysis
            dataset::get_dataset_stats,
            dataset::get_labelme_summary,
            // Directory operations
            directory::get_directory_images,
            directory::get_paginated_images,
            directory::get_image_details,
            directory::auto_annotate_images,
            // Advanced processing
            advanced::crop_and_remap_annotations,
            advanced::generate_annotated_previews,
            advanced::crop_remap_adapter,
            // LabelMe conversion commands
            commands::labelme_convert::convert_labelme,
            commands::labelme_convert::quick_convert_to_yolo,
            commands::labelme_convert::quick_convert_to_coco,
            commands::labelme_convert::scan_labelme_labels,
            commands::labelme_convert::scan_labelme_labels_with_counts,
            commands::labelme_convert::count_labelme_files,
            commands::labelme_convert::analyze_labelme_dataset,
            // 🆕 Async versions with progress reporting
            commands::labelme_convert::scan_labelme_labels_async,
            commands::labelme_convert::scan_labelme_labels_with_counts_async,
            commands::labelme_convert::analyze_labelme_dataset_async,
            // External module functions
            core::labelme2yolo::export_to_yolo_new,
            core::preview::generate_single_annotated_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

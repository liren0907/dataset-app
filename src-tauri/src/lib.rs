// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod commands;
pub mod core;
pub mod crop_remap;
pub mod labelme_convert;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init()) // Moved from main.rs to here
        .invoke_handler(tauri::generate_handler![
            // Media commands
            commands::media::get_video_info,
            commands::media::read_video_file,
            commands::media::read_image_file,
            // File operations
            commands::file_ops::read_file_content,
            commands::file_ops::open_file_dialog,
            // Drawing commands
            commands::drawing::draw_bounding_boxes,
            commands::drawing::draw_polygons,
            commands::drawing::visualize_dataset,
            // Export commands
            commands::export::export_to_yolo,
            commands::export::convert_to_labelme,
            // Label management
            commands::labels::change_label_name,
            commands::labels::clear_image_data,
            commands::labels::extract_labels,
            // Dataset analysis
            commands::dataset::get_dataset_stats,
            commands::dataset::get_labelme_summary,
            // Directory operations
            commands::directory::get_directory_images,
            commands::directory::get_paginated_images,
            commands::directory::get_image_details,
            commands::directory::auto_annotate_images,
            commands::dataset::save_annotation,
            // Advanced processing
            commands::advanced::crop_and_remap_annotations,
            commands::advanced::generate_annotated_previews,
            commands::advanced::crop_remap_adapter,
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

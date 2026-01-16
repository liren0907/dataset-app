use sentinelai_panel_lib::core::directory_handler::DirectoryHandler;
use sentinelai_panel_lib::core::image_annotator::ImageAnnotator;
use std::env;
use std::fs;

fn main() {
    println!("🚀 Starting Mock Data Generator...");

    // 1. Determine Paths
    // Assume we run this from the project root (dataset-app) or src-tauri
    // Standardize to project root
    let current_dir = env::current_dir().unwrap();
    println!("📂 Current Directory: {:?}", current_dir);

    // This path mimics what the user selects in the file dialog
    // In our repo, it's at dataset-app/mock_data/dataset_bounding_box
    // Adjust logic to find it
    let mock_data_path = if current_dir.join("mock_data").exists() {
        current_dir.join("mock_data").join("dataset_bounding_box")
    } else if current_dir.join("../mock_data").exists() {
        current_dir
            .join("../mock_data")
            .join("dataset_bounding_box")
    } else {
        panic!("❌ Could not find mock_data directory! Run this from project root.");
    };

    let mock_data_str = mock_data_path.to_str().expect("Invalid path string");
    println!("🎯 Target Data: {}", mock_data_str);

    // 2. Generate Image List (mimics get_paginated_images)
    println!("📸 Generating Image List...");
    // Fetch all (page 1, huge page size)
    let paginated_json_result = DirectoryHandler::get_paginated_images(mock_data_str, 1, 1000);

    let images_json = match paginated_json_result {
        Ok(json) => json,
        Err(e) => panic!("❌ Failed to get images: {}", e),
    };

    println!("✅ Generated Image List ({} bytes)", images_json.len());

    // 3. Generate Annotation Data (mimics auto_annotate_images)
    println!("🏷️ Generating Annotation Data...");
    let annotations_json_result = ImageAnnotator::auto_annotate_images(mock_data_str, 1, 1000);

    let annotations_json = match annotations_json_result {
        Ok(json) => json,
        Err(e) => panic!("❌ Failed to get annotations: {}", e),
    };

    println!(
        "✅ Generated Annotation Data ({} bytes)",
        annotations_json.len()
    );

    // 4. Write to src/mocks/
    // Target: src/mocks/generated/
    let output_dir = if current_dir.join("src").exists() {
        current_dir.join("src").join("mocks").join("generated")
    } else {
        // Fallback for running inside src-tauri
        current_dir.join("../src").join("mocks").join("generated")
    };

    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).expect("Failed to create output dir");
    }

    let images_out_path = output_dir.join("mock_images.json");
    let annotations_out_path = output_dir.join("mock_annotations.json");

    fs::write(&images_out_path, images_json).expect("Failed to write mock_images.json");
    fs::write(&annotations_out_path, annotations_json)
        .expect("Failed to write mock_annotations.json");

    println!("✨ Success! Mock data written to:");
    println!("   - {:?}", images_out_path);
    println!("   - {:?}", annotations_out_path);
}

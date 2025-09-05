use serde_json;
use std::fs;
use std::path::Path;

pub fn process_directories(root_dir: &str, target_dir: &str) -> Result<usize, String> {
    let root_path = Path::new(root_dir);

    if !root_path.is_dir() {
        return Err(format!("Root directory does not exist: {}", root_dir));
    }

    let image_extensions = vec!["jpg", "jpeg", "png", "gif", "bmp"];
    let annotation_extensions = vec!["json"];
    let mut image_count = 0;

    for entry in fs::read_dir(root_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if image_extensions
                    .contains(&extension.to_str().unwrap_or("").to_lowercase().as_str())
                {
                    image_count += 1;
                }
            }
        }
    }

    println!("Number of image files in root directory: {}", image_count);
    println!("Target directory: {}", target_dir);

    // TODO: Add logic for processing the target directory
    let target_path = Path::new(target_dir);

    if !target_path.is_dir() {
        return Err(format!("Target directory does not exist: {}", target_dir));
    }

    for entry in fs::read_dir(target_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if annotation_extensions
                    .contains(&extension.to_str().unwrap_or("").to_lowercase().as_str())
                {
                    let json_content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    match serde_json::from_str::<serde_json::Value>(&json_content) {
                        Ok(json_value) => {
                            if let Some(version) = json_value.get("version") {
                                println!("Version found in {}: {}", path.display(), version);
                            } else {
                                println!("No version key found in {}", path.display());
                            }
                        }
                        Err(e) => {
                            println!("Failed to parse JSON from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    Ok(image_count)
}

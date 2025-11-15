use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use image::GenericImageView;
use serde_json::json;
use serde_yaml;

/// Converter for transforming YOLO annotations to LabelMe format
pub struct YoloToLabelmeConverter {
    source_dir: String,
    output_dir: String,
    version: String,
    class_label_file: String,
    class_names: HashMap<usize, String>,
}

impl YoloToLabelmeConverter {
    /// Create a new YOLO to LabelMe converter
    pub fn new(source_dir: &str, output_dir: &str, version: &str, class_label_file: &str) -> Self {
        Self {
            source_dir: source_dir.to_string(),
            output_dir: output_dir.to_string(),
            version: version.to_string(),
            class_label_file: class_label_file.to_string(),
            class_names: HashMap::new(),
        }
    }

    /// Convert YOLO annotations to LabelMe format
    pub fn convert(&mut self) -> Result<(), Box<dyn Error>> {
        // Load class names from file
        self.load_class_names()?;

        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.output_dir)?;

        // Get all text annotation files in YOLO format
        let txt_files = self.find_yolo_annotations()?;

        // Process each annotation file
        for txt_file in txt_files {
            self.convert_annotation_file(&txt_file)?;
        }

        Ok(())
    }

    // Load class names from YAML or TXT file
    fn load_class_names(&mut self) -> Result<(), Box<dyn Error>> {
        let class_file_path = Path::new(&self.source_dir).join(&self.class_label_file);

        if !class_file_path.exists() {
            return Err(format!("Class label file not found: {}", class_file_path.display()).into());
        }

        let extension = class_file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension.to_lowercase().as_str() {
            "yaml" | "yml" => {
                // Parse YAML file
                let file = File::open(&class_file_path)?;
                let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;

                if let Some(names) = yaml["names"].as_sequence() {
                    for (idx, name) in names.iter().enumerate() {
                        if let Some(name_str) = name.as_str() {
                            self.class_names.insert(idx, name_str.to_string());
                        }
                    }
                } else {
                    return Err("Invalid YAML format: 'names' key not found or not a sequence".into());
                }
            },
            "txt" | "names" => {
                // Parse txt file with one class per line
                let file = File::open(&class_file_path)?;
                let reader = BufReader::new(file);

                for (idx, line) in reader.lines().enumerate() {
                    let name = line?;
                    if !name.trim().is_empty() {
                        self.class_names.insert(idx, name);
                    }
                }
            },
            _ => {
                return Err(format!("Unsupported class label file format: {}", extension).into());
            }
        }

        // Verify that we loaded at least one class
        if self.class_names.is_empty() {
            return Err("No class names loaded".into());
        }

        Ok(())
    }

    // Find all YOLO annotation files in the source directory
    fn find_yolo_annotations(&self) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut txt_files = Vec::new();
        let source_dir = Path::new(&self.source_dir);

        // Check if the source directory exists
        if !source_dir.exists() || !source_dir.is_dir() {
            return Err(format!("Source directory not found: {}", source_dir.display()).into());
        }

        // Walk through the directory and its subdirectories
        self.find_txt_files_recursive(source_dir, &mut txt_files)?;

        if txt_files.is_empty() {
            println!("Warning: No annotation files found in {}", source_dir.display());
        }

        Ok(txt_files)
    }

    // Recursively find all .txt files in a directory and its subdirectories
    fn find_txt_files_recursive(&self, dir: &Path, txt_files: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    self.find_txt_files_recursive(&path, txt_files)?;
                } else if path.extension().map_or(false, |ext| ext == "txt") {
                    let file_stem = path.file_stem().unwrap().to_str().unwrap();
                    // Skip dataset.txt or classes.txt files
                    if !file_stem.starts_with("dataset") && !file_stem.starts_with("classes") {
                        txt_files.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    // Convert a single YOLO annotation file to LabelMe JSON
    fn convert_annotation_file(&self, txt_file: &Path) -> Result<(), Box<dyn Error>> {
        // Find corresponding image file
        let img_path = self.find_matching_image(txt_file)?;
        if !img_path.exists() {
            return Err(format!("Image file not found for annotation: {}", txt_file.display()).into());
        }

        // Get image dimensions
        let img = image::open(&img_path)?;
        let (img_width, img_height) = img.dimensions();

        // Read YOLO annotation file
        let file = File::open(txt_file)?;
        let reader = BufReader::new(file);
        let mut shapes = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.trim().split_whitespace().collect();

            if parts.is_empty() || parts[0].starts_with("#") {
                // Skip empty lines and comments
                continue;
            }

            if parts.len() >= 5 {
                // Process bounding box format: class_id x_center y_center width height
                let class_id: usize = parts[0].parse()?;
                let x_center: f32 = parts[1].parse()?;
                let y_center: f32 = parts[2].parse()?;
                let width: f32 = parts[3].parse()?;
                let height: f32 = parts[4].parse()?;

                // Convert normalized coordinates to pixel coordinates
                let x1 = ((x_center - width / 2.0) * img_width as f32) as i32;
                let y1 = ((y_center - height / 2.0) * img_height as f32) as i32;
                let x2 = ((x_center + width / 2.0) * img_width as f32) as i32;
                let y2 = ((y_center + height / 2.0) * img_height as f32) as i32;

                // Get class name
                let label = self.class_names.get(&class_id)
                    .cloned()
                    .unwrap_or_else(|| format!("class_{}", class_id));

                // Create LabelMe shape
                let shape = json!({
                    "label": label,
                    "group_id": null,
                    "shape_type": "rectangle",
                    "flags": {},
                    "points": [
                        [x1, y1],
                        [x2, y2]
                    ]
                });

                shapes.push(shape);
            } else if parts.len() > 5 && parts.len() % 2 == 1 {
                // Process polygon format: class_id x1 y1 x2 y2 ... xn yn
                let class_id: usize = parts[0].parse()?;
                let mut points = Vec::new();

                for i in (1..parts.len()).step_by(2) {
                    if i + 1 < parts.len() {
                        let x: f32 = parts[i].parse()?;
                        let y: f32 = parts[i + 1].parse()?;

                        // Convert normalized coordinates to pixel coordinates
                        let px = (x * img_width as f32) as i32;
                        let py = (y * img_height as f32) as i32;

                        points.push(json!([px, py]));
                    }
                }

                // Get class name
                let label = self.class_names.get(&class_id)
                    .cloned()
                    .unwrap_or_else(|| format!("class_{}", class_id));

                // Create LabelMe shape
                let shape = json!({
                    "label": label,
                    "group_id": null,
                    "shape_type": "polygon",
                    "flags": {},
                    "points": points
                });

                shapes.push(shape);
            }
        }

        // Create LabelMe JSON
        let relative_image_path = img_path.file_name().unwrap().to_str().unwrap();
        let labelme_json = json!({
            "version": self.version,
            "flags": {},
            "shapes": shapes,
            "imagePath": relative_image_path,
            "imageData": null,
            "imageHeight": img_height,
            "imageWidth": img_width
        });

        // Save LabelMe JSON
        let output_json_name = format!("{}.json", txt_file.file_stem().unwrap().to_str().unwrap());
        let output_json_path = Path::new(&self.output_dir).join(output_json_name);
        fs::write(
            output_json_path,
            serde_json::to_string_pretty(&labelme_json)?
        )?;

        // Copy image to output directory
        let output_image_path = Path::new(&self.output_dir).join(relative_image_path);
        fs::copy(img_path, output_image_path)?;

        Ok(())
    }

    // Find matching image file for a YOLO annotation file
    fn find_matching_image(&self, txt_file: &Path) -> Result<PathBuf, Box<dyn Error>> {
        let file_stem = txt_file.file_stem().unwrap().to_str().unwrap();
        let parent_dir = txt_file.parent().unwrap_or_else(|| Path::new(""));

        // Try common image extensions
        for ext in &["jpg", "jpeg", "png", "bmp", "tif", "tiff"] {
            let img_path = parent_dir.join(format!("{}.{}", file_stem, ext));
            if img_path.exists() {
                return Ok(img_path);
            }
        }

        // Look for images in a parallel "images" directory
        if let Some(parent_of_parent) = parent_dir.parent() {
            let images_dir = parent_of_parent.join("images");
            if images_dir.exists() && images_dir.is_dir() {
                for ext in &["jpg", "jpeg", "png", "bmp", "tif", "tiff"] {
                    let img_path = images_dir.join(format!("{}.{}", file_stem, ext));
                    if img_path.exists() {
                        return Ok(img_path);
                    }
                }
            }
        }

        // If we haven't found anything, try to deduce from directory structure
        // YOLO format often uses a structure like:
        // - labels/train/xxx.txt
        // - images/train/xxx.jpg
        let path_str = txt_file.to_str().unwrap();
        if path_str.contains("labels/") || path_str.contains("labels\\") {
            let alt_path = path_str.replace("labels/", "images/").replace("labels\\", "images\\");
            let alt_dir = Path::new(&alt_path).parent().unwrap();

            for ext in &["jpg", "jpeg", "png", "bmp", "tif", "tiff"] {
                let img_path = alt_dir.join(format!("{}.{}", file_stem, ext));
                if img_path.exists() {
                    return Ok(img_path);
                }
            }
        }

        Err(format!("No matching image found for {}", txt_file.display()).into())
    }
}

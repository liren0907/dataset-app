use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::{self, Value};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Exporter for converting LabelMe annotations to YOLO format
pub struct YoloExporter {
    source_dir: String,
    output_dir: String,
    train_ratio: f32,
    val_ratio: f32,
    #[allow(dead_code)]
    test_ratio: f32,
    specific_labels: bool,
    class_names: Vec<String>,
    shape: String,
}

impl YoloExporter {
    /// Create a new YOLO exporter
    pub fn new(
        source_dir: &str,
        output_dir: &str,
        train_ratio: f32,
        val_ratio: f32,
        test_ratio: f32,
        shape: &str,
        specific_labels: bool,
        class_names: Vec<String>,
    ) -> Self {
        Self {
            source_dir: source_dir.to_string(),
            output_dir: output_dir.to_string(),
            train_ratio,
            val_ratio,
            test_ratio,
            specific_labels,
            class_names,
            shape: shape.to_string(),
        }
    }

    /// Export LabelMe annotations to YOLO format
    pub fn export(&self) -> Result<(), Box<dyn Error>> {
        // Create output directory structure
        let output_dir = Path::new(&self.output_dir);

        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        // Create subdirectories
        let images_dir = output_dir.join("images");
        let labels_dir = output_dir.join("labels");

        // Create train, val, test dirs
        for dir in &["train", "val", "test"] {
            fs::create_dir_all(images_dir.join(dir))?;
            fs::create_dir_all(labels_dir.join(dir))?;
        }

        // Get all JSON files in source directory
        let mut json_files = Vec::new();
        self.collect_json_files(Path::new(&self.source_dir), &mut json_files)?;

        // Generate class mapping
        let class_map = self.generate_class_map(&json_files)?;

        // Save class names to dataset.yaml file
        self.save_dataset_yaml(output_dir, &class_map)?;

        // Shuffle files for random split
        let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility
        json_files.shuffle(&mut rng);

        // Calculate split counts
        let total_files = json_files.len();
        let train_count = (total_files as f32 * self.train_ratio).round() as usize;
        let val_count = (total_files as f32 * self.val_ratio).round() as usize;

        // Split files
        let train_files = &json_files[0..train_count];
        let val_files = &json_files[train_count..train_count + val_count];
        let test_files = &json_files[train_count + val_count..];

        // Process each split
        self.process_split("train", train_files, &class_map, &images_dir, &labels_dir)?;
        self.process_split("val", val_files, &class_map, &images_dir, &labels_dir)?;
        self.process_split("test", test_files, &class_map, &images_dir, &labels_dir)?;

        Ok(())
    }

    // Helper to collect JSON files from directory and subdirectories
    fn collect_json_files(&self, dir: &Path, json_files: &mut Vec<PathBuf>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    self.collect_json_files(&path, json_files)?;
                } else if path.extension().map_or(false, |ext| ext == "json") {
                    json_files.push(path);
                }
            }
        }
        Ok(())
    }

    // Generate a mapping from class names to YOLO class indices
    fn generate_class_map(
        &self,
        json_files: &[PathBuf],
    ) -> Result<HashMap<String, usize>, Box<dyn Error>> {
        let mut class_map = HashMap::new();

        // If specific labels are provided, use them
        if self.specific_labels {
            for (i, name) in self.class_names.iter().enumerate() {
                class_map.insert(name.clone(), i);
            }
            return Ok(class_map);
        }

        // Otherwise, extract all unique labels from the dataset
        let mut classes = HashSet::new();

        for json_path in json_files {
            let json_content = fs::read_to_string(json_path)?;
            let json: Value = serde_json::from_str(&json_content)?;

            if let Some(shapes) = json["shapes"].as_array() {
                for shape in shapes {
                    if let Some(label) = shape["label"].as_str() {
                        classes.insert(label.to_string());
                    }
                }
            }
        }

        // Sort classes alphabetically for consistent ordering
        let mut sorted_classes: Vec<String> = classes.into_iter().collect();
        sorted_classes.sort();

        // Create map with indices
        for (i, name) in sorted_classes.iter().enumerate() {
            class_map.insert(name.clone(), i);
        }

        Ok(class_map)
    }

    // Save dataset.yaml file for YOLOv5 training
    fn save_dataset_yaml(
        &self,
        output_dir: &Path,
        class_map: &HashMap<String, usize>,
    ) -> io::Result<()> {
        let yaml_path = output_dir.join("dataset.yaml");
        let mut file = BufWriter::new(File::create(yaml_path)?);

        // Create sorted classes vector
        let mut classes: Vec<(String, usize)> =
            class_map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        classes.sort_by_key(|(_, idx)| *idx);

        // Write header
        writeln!(file, "# YOLO dataset configuration")?;
        writeln!(file, "path: {}", self.output_dir)?;
        writeln!(file, "train: images/train")?;
        writeln!(file, "val: images/val")?;
        writeln!(file, "test: images/test")?;

        // Write class names
        writeln!(file, "nc: {}", classes.len())?;

        // Write class names array
        write!(file, "names: [")?;
        for (i, (name, _)) in classes.iter().enumerate() {
            if i > 0 {
                write!(file, ", ")?;
            }
            write!(file, "'{}'", name)?;
        }
        writeln!(file, "]")?;

        Ok(())
    }

    // Process a split of files (train, val, or test)
    fn process_split(
        &self,
        split: &str,
        files: &[PathBuf],
        class_map: &HashMap<String, usize>,
        images_dir: &Path,
        labels_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        for json_path in files {
            self.convert_file(json_path, split, class_map, images_dir, labels_dir)?;
        }
        Ok(())
    }

    // Convert a single LabelMe JSON file to YOLO format
    fn convert_file(
        &self,
        json_path: &Path,
        split: &str,
        class_map: &HashMap<String, usize>,
        images_dir: &Path,
        labels_dir: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // Read and parse JSON
        let json_content = fs::read_to_string(json_path)?;
        let json: Value = serde_json::from_str(&json_content)?;

        // Get image path and dimensions
        let image_path_str = json["imagePath"].as_str().ok_or("Missing imagePath")?;
        let img_width = json["imageWidth"].as_f64().ok_or("Missing imageWidth")? as f32;
        let img_height = json["imageHeight"].as_f64().ok_or("Missing imageHeight")? as f32;

        // Find image file relative to JSON file's directory if possible, else relative to source_dir
        let source_image = match json_path.parent() {
            Some(parent_dir) => parent_dir.join(image_path_str),
            None => Path::new(&self.source_dir).join(image_path_str),
        };

        if !source_image.exists() {
            // Try relative to the main source directory as a fallback
            let fallback_source_image = Path::new(&self.source_dir).join(image_path_str);
            if !fallback_source_image.exists() {
                println!(
                    "Warning: Image not found at {:?} or {:?}. Skipping.",
                    source_image, fallback_source_image
                );
                return Ok(()); // Skip this file if image not found
            }
            // Use fallback path if it exists
            fs::copy(
                fallback_source_image,
                images_dir
                    .join(split)
                    .join(source_image.file_name().unwrap()),
            )?;
        } else {
            // Use original path
            fs::copy(
                &source_image,
                images_dir
                    .join(split)
                    .join(source_image.file_name().unwrap()),
            )?;
        }

        // Create YOLO label file path
        let label_filename = format!("{}.txt", json_path.file_stem().unwrap().to_str().unwrap());
        let label_path = labels_dir.join(split).join(label_filename);
        let mut label_file = BufWriter::new(File::create(label_path)?);

        // Process shapes INDIVIDUALLY
        if let Some(shapes) = json["shapes"].as_array() {
            for shape in shapes {
                // Iterate through each shape in the JSON
                let label_opt = shape["label"].as_str();
                let shape_type_opt = shape["shape_type"].as_str();
                let points_opt = shape["points"].as_array();

                // Ensure we have the essential parts
                if let (Some(label), Some(shape_type), Some(points)) =
                    (label_opt, shape_type_opt, points_opt)
                {
                    // --- Filtering Logic ---
                    // Skip if we're using specific labels and this one isn't in the list
                    if self.specific_labels && !self.class_names.contains(&label.to_string()) {
                        continue; // Skip this shape
                    }

                    // Get class index
                    let class_idx = match class_map.get(label) {
                        Some(idx) => *idx,
                        None => {
                            // println!("Warning: Label '{}' not in class map for shape in {:?}, skipping shape", label, json_path);
                            continue; // Skip shape if label not in map
                        }
                    };

                    // --- Coordinate Calculation and Writing Logic ---
                    // Bounding Box Output:
                    if self.shape == "bounding_box" {
                        // Can generate a bbox from rectangle, polygon, or potentially others if they have points
                        if shape_type == "rectangle"
                            || shape_type == "polygon"
                            || shape_type == "bounding_box"
                        {
                            let mut min_x = f32::MAX;
                            let mut min_y = f32::MAX;
                            let mut max_x = f32::MIN;
                            let mut max_y = f32::MIN;
                            let mut point_count = 0;

                            for point_val in points {
                                if let (Some(x), Some(y)) =
                                    (point_val[0].as_f64(), point_val[1].as_f64())
                                {
                                    min_x = min_x.min(x as f32);
                                    min_y = min_y.min(y as f32);
                                    max_x = max_x.max(x as f32);
                                    max_y = max_y.max(y as f32);
                                    point_count += 1;
                                }
                            }

                            // Ensure valid points were found and box has area
                            if point_count >= 2 && max_x > min_x && max_y > min_y {
                                // Convert to YOLO format (normalized)
                                let x_center = (min_x + max_x) / 2.0 / img_width;
                                let y_center = (min_y + max_y) / 2.0 / img_height;
                                let width = (max_x - min_x) / img_width;
                                let height = (max_y - min_y) / img_height;

                                // Write ONE line for THIS shape
                                writeln!(
                                    label_file,
                                    "{} {:.6} {:.6} {:.6} {:.6}",
                                    class_idx, x_center, y_center, width, height
                                )?;
                            } else {
                                // println!("Warning: Skipping shape with label '{}' in {:?} due to invalid points or zero area for bbox.", label, json_path);
                            }
                        } else {
                            // println!("Warning: Skipping shape with label '{}' and type '{}' in {:?} as it cannot be used for bbox output.", label, shape_type, json_path);
                        }
                    }
                    // Polygon Output:
                    else if self.shape == "polygon" {
                        // Only process shapes explicitly marked as polygon
                        if shape_type == "polygon" {
                            let mut normalized_points: Vec<(f32, f32)> = Vec::new();
                            for point_val in points {
                                if let (Some(x), Some(y)) =
                                    (point_val[0].as_f64(), point_val[1].as_f64())
                                {
                                    normalized_points
                                        .push((x as f32 / img_width, y as f32 / img_height));
                                }
                            }

                            // Ensure we have enough points for a polygon
                            if normalized_points.len() >= 3 {
                                // Write class index
                                write!(label_file, "{}", class_idx)?;
                                // Write each point (normalized)
                                for (norm_x, norm_y) in normalized_points {
                                    write!(label_file, " {:.6} {:.6}", norm_x, norm_y)?;
                                }
                                // End the line for THIS polygon shape
                                writeln!(label_file)?;
                            } else {
                                // println!("Warning: Skipping polygon shape with label '{}' in {:?} due to insufficient points (need >= 3).", label, json_path);
                            }
                        } else {
                            // println!("Warning: Skipping shape with label '{}' and type '{}' in {:?} for polygon output (requires 'polygon' shape_type).", label, shape_type, json_path);
                        }
                    }
                } else {
                    // println!("Warning: Skipping shape in {:?} due to missing label, shape_type, or points.", json_path);
                }
            } // End loop through individual shapes
        }

        Ok(())
    }
}

#[allow(dead_code)]
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
pub fn export_to_yolo_new(
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

    let base_output_path = PathBuf::from(&output_dir);
    let final_output_path;

    // Determine the final output path based on whether the base_output_path exists
    if base_output_path.exists() {
        // If output_dir exists, create and use a 'yolo_output' subdirectory within it
        final_output_path = base_output_path.join("yolo_output");
    } else {
        // If output_dir does not exist, use it directly as the final path
        final_output_path = base_output_path;
    }

    // Create the final output directory (and any parent directories) if they don't exist
    if let Err(e) = std::fs::create_dir_all(&final_output_path) {
        return Err(format!(
            "Failed to create output directory {}: {}",
            final_output_path.display(),
            e
        ));
    }

    // Create exporter
    let exporter = YoloExporter::new(
        &source_dir,
        final_output_path
            .to_str()
            .ok_or("Invalid final output path string")?,
        train_ratio,
        val_ratio,
        test_ratio,
        &shape,
        specific_labels,
        class_names,
    );

    // Export
    match exporter.export() {
        Ok(_) => {
            Ok("Dataset exported to YOLO format successfully (using new function)".to_string())
        }
        Err(e) => Err(format!(
            "Failed to export to YOLO format (using new function): {}",
            e
        )),
    }
}

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::error::Error;
use serde_json::{Value, json};
use prettytable::{Table, row};

pub struct LabelmeViewerModule {
    source_directory: String,
    label_counter: HashMap<String, usize>,
    shape_type_counter: HashMap<String, HashMap<String, usize>>,
}

impl LabelmeViewerModule {
    pub fn new(source_directory: &str) -> Result<Self, Box<dyn Error>> {
        if !Path::new(source_directory).exists() {
            return Err("Source directory does not exist".into());
        }
        
        Ok(Self {
            source_directory: source_directory.to_string(),
            label_counter: HashMap::new(),
            shape_type_counter: HashMap::new(),
        })
    }
    
    pub fn get_source_directory(&self) -> &str {
        &self.source_directory
    }

    pub fn summary(&mut self) {
        let mut total_json_files = 0;
        let mut total_image_files = 0;
        let image_extensions = vec!["jpg", "jpeg", "png", "gif", "bmp", "tiff"];

        if let Ok(entries) = fs::read_dir(&self.source_directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().is_some() {
                        let ext = path.extension().unwrap().to_str().unwrap().to_lowercase();
                        if ext == "json" {
                            total_json_files += 1;
                            self.summarize_annotation(&path);
                        } else if image_extensions.contains(&ext.as_str()) {
                            total_image_files += 1;
                        }
                    }
                }
            }
        }

        println!("Total JSON files: {}", total_json_files);
        println!("Total image files: {}", total_image_files);

        // Print label statistics
        println!("Label Statistics:");
        for (label, count) in &self.label_counter {
            println!("Label: {}, Count: {}", label, count);
        }

    

        let mut table = Table::new();
        table.add_row(row!["Label", "Count"]);

        for (label, count) in &self.label_counter {
            table.add_row(row![label, count]);
        }

        table.printstd();
    }

    fn summarize_annotation(&mut self, json_file_path: &Path) {
        if let Ok(file) = fs::read_to_string(json_file_path) {
            if let Ok(data) = serde_json::from_str::<Value>(&file) {
                if let Some(shapes) = data.get("shapes").and_then(|s| s.as_array()) {
                    for shape in shapes {
                        if let Some(label) = shape.get("label").and_then(|l| l.as_str()) {
                            *self.label_counter.entry(label.to_string()).or_insert(0) += 1;
                            if let Some(shape_type) = shape.get("shape_type").and_then(|st| st.as_str()) {
                                let shape_counter = self.shape_type_counter.entry(label.to_string()).or_insert_with(HashMap::new);
                                *shape_counter.entry(shape_type.to_string()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Change label name in all annotation files
    pub fn change_label_name(&self, output_directory: &str, old_label: &str, new_label: &str) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(output_directory)?;

        if let Ok(entries) = fs::read_dir(&self.source_directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "json") {
                        let content = fs::read_to_string(&path)?;
                        let mut json: Value = serde_json::from_str(&content)?;

                        // Process shapes array
                        if let Some(shapes) = json["shapes"].as_array_mut() {
                            let mut modified = false;
                            for shape in shapes {
                                if let Some(label) = shape["label"].as_str() {
                                    if label == old_label {
                                        shape["label"] = json!(new_label);
                                        modified = true;
                                    }
                                }
                            }

                            // Only save if modified
                            if modified {
                                let output_path = Path::new(output_directory).join(path.file_name().unwrap());
                                fs::write(output_path, serde_json::to_string_pretty(&json)?)?;
                                
                                // Copy image if it exists
                                if let Some(image_path) = json["imagePath"].as_str() {
                                    let source_image_path = Path::new(&self.source_directory).join(image_path);
                                    let dest_image_path = Path::new(output_directory).join(image_path);
                                    
                                    if source_image_path.exists() {
                                        fs::copy(source_image_path, dest_image_path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
    
    /// Remove imageData from all annotation files
    pub fn clear_image_data(&self, output_directory: &str) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(output_directory)?;

        if let Ok(entries) = fs::read_dir(&self.source_directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "json") {
                        let content = fs::read_to_string(&path)?;
                        let mut json: Value = serde_json::from_str(&content)?;

                        if let Some(obj) = json.as_object_mut() {
                            obj.insert("imageData".to_string(), Value::Null);
                        }

                        let output_path = Path::new(output_directory).join(path.file_name().unwrap());
                        fs::write(output_path, serde_json::to_string_pretty(&json)?)?;
                    }
                }
            }
        }

        Ok(())
    }
    
    /// Extract labels from annotation files
    pub fn extract_labels(&self, output_directory: &str, labels_to_extract: &[String]) -> io::Result<()> {
        fs::create_dir_all(output_directory)?;

        if let Ok(entries) = fs::read_dir(&self.source_directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "json") {
                        let content = fs::read_to_string(&path)?;
                        
                        if let Ok(mut json) = serde_json::from_str::<Value>(&content) {
                            if let Some(shapes) = json["shapes"].as_array() {
                                // Check if any shape has labels we want to extract
                                let extract_file = shapes.iter().any(|shape| {
                                    if let Some(label) = shape["label"].as_str() {
                                        labels_to_extract.contains(&label.to_string())
                                    } else {
                                        false
                                    }
                                });
                                
                                if extract_file {
                                    // Filter shapes to include only those with matching labels
                                    if let Some(shapes_mut) = json["shapes"].as_array_mut() {
                                        let filtered_shapes: Vec<_> = shapes_mut
                                            .iter()
                                            .filter(|shape| {
                                                if let Some(label) = shape["label"].as_str() {
                                                    labels_to_extract.contains(&label.to_string())
                                                } else {
                                                    false
                                                }
                                            })
                                            .cloned()
                                            .collect();
                                        
                                        *shapes_mut = json!(filtered_shapes).as_array().unwrap().clone();
                                    }
                                    
                                    // Save filtered JSON
                                    let output_path = Path::new(output_directory).join(path.file_name().unwrap());
                                    fs::write(output_path, serde_json::to_string_pretty(&json)?)?;
                                    
                                    // Copy image if available
                                    if let Some(image_path) = json["imagePath"].as_str() {
                                        let source_image_path = Path::new(&self.source_directory).join(image_path);
                                        let dest_image_path = Path::new(output_directory).join(image_path);
                                        
                                        if source_image_path.exists() {
                                            fs::copy(source_image_path, dest_image_path)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
    
    /// Get detailed statistics about the dataset
    pub fn get_statistics(&mut self) -> HashMap<String, HashMap<String, usize>> {
        // Reset counters
        self.label_counter.clear();
        self.shape_type_counter.clear();
        
        if let Ok(entries) = fs::read_dir(&self.source_directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "json") {
                        self.summarize_annotation(&path);
                    }
                }
            }
        }
        
        self.shape_type_counter.clone()
    }
}
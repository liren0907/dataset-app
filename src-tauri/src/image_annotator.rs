use serde::{Serialize, Deserialize};
use std::path::{Path, /* PathBuf */};
use crate::directory_handler::{DirectoryHandler, PaginatedResult};
// use glob::glob;
use std::fs;
// use crate::polygon_drawer;
// use crate::bounding_box_drawer;
// use crate::labelme_types::{LabelMeFile, LabelMeShape};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotation {
    pub label: String,
    pub confidence: f32,
    pub shape_type: String,
    pub points: Option<Vec<Vec<f32>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotatedImage {
    pub path: String, 
    pub annotations: Vec<Annotation>,
    pub has_json: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationResult {
    pub annotated_images: Vec<AnnotatedImage>,
    pub total: usize,
}

pub struct ImageAnnotator;

impl ImageAnnotator {
    pub fn auto_annotate_images(path: &str, page: usize, page_size: usize, annotationType: &str) -> Result<String, String> {
        // Validate annotation type
        println!("Starting annotation with type: {}", annotationType);
        if annotationType != "bounding_box" && annotationType != "polygon" {
            return Err(format!("Invalid annotation type: {}. Must be 'bounding_box' or 'polygon'", annotationType));
        }
        
        // Get images for the current page from the directory handler
        println!("Loading images from: {}, page: {}, page_size: {}", path, page, page_size);
        let paginated_result = DirectoryHandler::get_paginated_images(path, page, page_size)?;
        
        // Parse the result to get the images
        let parsed_result: PaginatedResult = serde_json::from_str(&paginated_result)
            .map_err(|e| format!("Failed to parse paginated result: {}", e))?;
        
        println!("Found {} images in page {} of {}", parsed_result.images.len(), page, parsed_result.total_pages);
        
        // Process each image to find and parse its annotation file
        let mut annotated_images: Vec<AnnotatedImage> = Vec::new();
        
        for image in parsed_result.images {
            println!("Processing image: {}", image.path);
            // Find corresponding JSON annotation file
            let annotations = Self::find_annotations(&image.path, annotationType)?;
            
            // Add to annotated images
            let has_json = !annotations.is_empty();
            if has_json {
                println!("Found {} annotations for {}", annotations.len(), image.path);
            }
            
            annotated_images.push(AnnotatedImage {
                path: image.path,
                annotations,
                has_json,
            });
        }
        
        // Count JSON files and annotations
        let with_json = annotated_images.iter().filter(|img| img.has_json).count();
        let with_annotations = annotated_images.iter().filter(|img| !img.annotations.is_empty()).count();
        println!("Processed {} images: {} with JSON files, {} with {} annotations", 
            annotated_images.len(), with_json, with_annotations, annotationType);
        
        // Create the result
        let result = AnnotationResult {
            annotated_images,
            total: parsed_result.total,
        };
        
        // Serialize the result
        match serde_json::to_string(&result) {
            Ok(json) => {
                println!("Successfully serialized annotation result");
                Ok(json)
            },
            Err(e) => Err(format!("Failed to serialize annotation result: {}", e)),
        }
    }
    
    // Find and parse LabelMe JSON annotation file for an image
    fn find_annotations(image_path: &str, annotationType: &str) -> Result<Vec<Annotation>, String> {
        let image_file_path = Path::new(image_path);
        
        // Get image filename without extension
        let file_stem = match image_file_path.file_stem() {
            Some(stem) => stem.to_string_lossy().to_string(),
            None => return Ok(Vec::new()), // No filename found
        };
        
        // Get directory containing the image
        let parent_dir = match image_file_path.parent() {
            Some(dir) => dir,
            None => return Ok(Vec::new()), // No parent directory found
        };
        
        // Look for JSON with same name as image
        let json_path = parent_dir.join(format!("{}.json", file_stem));
        
        if !json_path.exists() {
            return Ok(Vec::new()); // No JSON file found
        }
        
        println!("Found JSON file for {}: {:?}", image_path, json_path);
        
        // Read and parse the JSON file
        let json_content = match fs::read_to_string(&json_path) {
            Ok(content) => content,
            Err(e) => {
                println!("Error reading JSON file: {}", e);
                return Ok(Vec::new());
            },
        };
        
        let json_value: serde_json::Value = match serde_json::from_str(&json_content) {
            Ok(parsed) => parsed,
            Err(e) => {
                println!("Error parsing JSON: {}", e);
                return Ok(Vec::new());
            },
        };
        
        // Extract shapes from LabelMe format
        let shapes = match json_value.get("shapes") {
            Some(s) => match s.as_array() {
                Some(array) => {
                    println!("Found {} shapes in JSON", array.len());
                    array
                },
                None => {
                    println!("Shapes is not an array");
                    return Ok(Vec::new());
                },
            },
            None => {
                println!("No shapes found in JSON");
                return Ok(Vec::new());
            },
        };
        
        // Convert shapes to our Annotation format
        let mut annotations = Vec::new();
        
        for shape in shapes {
            // Only process shapes matching the requested type
            let shape_type = match shape.get("shape_type") {
                Some(st) => match st.as_str() {
                    Some(s) => s,
                    None => continue, // shape_type is not a string
                },
                None => continue, // No shape_type found
            };
            
            // Filter by annotation type
            let is_valid_type = match annotationType {
                "bounding_box" => shape_type == "rectangle",
                "polygon" => shape_type == "polygon",
                _ => false,
            };
            
            if !is_valid_type {
                println!("Skipping shape of type {}, wanted {}", shape_type, annotationType);
                continue;
            }
            
            // Get label
            let label = match shape.get("label") {
                Some(l) => match l.as_str() {
                    Some(s) => s.to_string(),
                    None => "unknown".to_string(),
                },
                None => "unknown".to_string(),
            };
            
            println!("Processing {} annotation with label: {}", shape_type, label);
            
            // Get points (coordinates)
            let points_json = match shape.get("points") {
                Some(p) => p,
                None => continue, // No points found
            };
            
            let points_array = match points_json.as_array() {
                Some(arr) => arr,
                None => continue, // points is not an array
            };
            
            // Convert points from LabelMe format
            let mut points: Vec<Vec<f32>> = Vec::new();
            for point in points_array {
                if let Some(coords) = point.as_array() {
                    if coords.len() >= 2 {
                        if let (Some(x), Some(y)) = (coords[0].as_f64(), coords[1].as_f64()) {
                            points.push(vec![x as f32, y as f32]);
                        }
                    }
                }
            }
            
            println!("Added annotation with {} points", points.len());
            
            // Add to annotations
            annotations.push(Annotation {
                label,
                confidence: 1.0, // 100% confidence for manually created annotations
                shape_type: shape_type.to_string(),
                points: Some(points),
            });
        }
        
        println!("Found {} matching annotations of type {}", annotations.len(), annotationType);
        Ok(annotations)
    }
} 
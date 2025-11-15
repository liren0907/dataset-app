use chrono::{DateTime, Utc};
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

// Cache for directory contents to avoid rescanning
lazy_static::lazy_static! {
    static ref DIRECTORY_CACHE: Arc<Mutex<HashMap<String, Vec<ImageFile>>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageFile {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub dimensions: Option<ImageDimensions>,
    pub created: Option<i64>,
    pub modified: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult {
    pub images: Vec<ImageFile>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

pub struct DirectoryHandler;

impl DirectoryHandler {
    pub fn get_directory_images(path: &str) -> Result<String, String> {
        let dir_path = Path::new(path);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(format!("Invalid directory path: {}", path));
        }

        let mut images = Vec::new();
        let image_extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".tiff", ".tif",
        ];

        // Walk through the directory and its subdirectories
        for entry in WalkDir::new(dir_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check if the file has an image extension
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if image_extensions
                    .iter()
                    .any(|&e| e.contains(&ext) || e[1..].contains(&ext))
                {
                    // Get file metadata
                    if let Ok(metadata) = fs::metadata(path) {
                        let size = metadata.len();

                        // Get creation and modification times
                        let created = metadata
                            .created()
                            .ok()
                            .and_then(|time| DateTime::<Utc>::from(time).timestamp_millis().into());

                        let modified = metadata
                            .modified()
                            .ok()
                            .and_then(|time| DateTime::<Utc>::from(time).timestamp_millis().into());

                        // Get image dimensions if possible
                        let dimensions = Self::get_image_dimensions(path);

                        // Create image file struct
                        let image = ImageFile {
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            path: path.to_string_lossy().to_string(),
                            size,
                            dimensions,
                            created,
                            modified,
                        };

                        images.push(image);
                    }
                }
            }
        }

        // Sort images by name
        images.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        // Serialize to JSON
        match serde_json::to_string(&images) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Failed to serialize image data: {}", e)),
        }
    }

    // New optimized function for paginated results
    pub fn get_paginated_images(
        path: &str,
        page: usize,
        page_size: usize,
    ) -> Result<String, String> {
        let dir_path = Path::new(path);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(format!("Invalid directory path: {}", path));
        }

        // Try to get images from cache first
        let path_key = path.to_string();
        let mut cache = DIRECTORY_CACHE.lock().unwrap();

        let all_images = if let Some(cached_images) = cache.get(&path_key) {
            // Use cached results
            cached_images.clone()
        } else {
            // Not in cache, scan directory
            let mut images = Vec::new();
            let image_extensions = [
                ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".webp", ".tiff", ".tif",
            ];

            // Walk through the directory
            for entry in WalkDir::new(dir_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                // Skip directories
                if path.is_dir() {
                    continue;
                }

                // Check if file has an image extension
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if image_extensions
                        .iter()
                        .any(|&e| e.contains(&ext) || e[1..].contains(&ext))
                    {
                        if let Ok(metadata) = fs::metadata(path) {
                            let size = metadata.len();

                            // Basic metadata only - defer loading dimensions until needed
                            let image = ImageFile {
                                name: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                path: path.to_string_lossy().to_string(),
                                size,
                                dimensions: None, // Don't load dimensions initially
                                created: metadata.created().ok().and_then(|time| {
                                    DateTime::<Utc>::from(time).timestamp_millis().into()
                                }),
                                modified: metadata.modified().ok().and_then(|time| {
                                    DateTime::<Utc>::from(time).timestamp_millis().into()
                                }),
                            };

                            images.push(image);
                        }
                    }
                }
            }

            // Sort images by name
            images.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // Store in cache
            cache.insert(path_key.clone(), images.clone());

            images
        };

        // Calculate pagination values
        let total = all_images.len();
        let start_index = page.saturating_sub(1) * page_size;
        let end_index = (start_index + page_size).min(total);
        let total_pages = (total + page_size - 1) / page_size;

        // Get slice for current page
        let page_images = if start_index < total {
            all_images[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };

        // Create paginated result
        let result = PaginatedResult {
            images: page_images,
            total,
            page,
            page_size,
            total_pages,
        };

        // Serialize to JSON
        match serde_json::to_string(&result) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Failed to serialize paginated data: {}", e)),
        }
    }

    // Get image details including dimensions
    pub fn get_image_details(path: &str) -> Result<String, String> {
        let file_path = Path::new(path);
        if !file_path.exists() || !file_path.is_file() {
            return Err(format!("Invalid image path: {}", path));
        }

        if let Ok(metadata) = fs::metadata(file_path) {
            let dimensions = Self::get_image_dimensions(file_path);

            let image = ImageFile {
                name: file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                path: path.to_string(),
                size: metadata.len(),
                dimensions,
                created: metadata
                    .created()
                    .ok()
                    .and_then(|time| DateTime::<Utc>::from(time).timestamp_millis().into()),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|time| DateTime::<Utc>::from(time).timestamp_millis().into()),
            };

            match serde_json::to_string(&image) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize image details: {}", e)),
            }
        } else {
            Err(format!("Failed to get metadata for: {}", path))
        }
    }

    fn get_image_dimensions(path: &Path) -> Option<ImageDimensions> {
        // Open the image
        image::open(path).ok().map(|img| {
            // Use GenericImageView to get dimensions
            let (width, height) = img.dimensions();
            ImageDimensions { width, height }
        })
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AutoDetectRequest {
    pub image_path: String,
    pub confidence_threshold: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug, Serialize)]
pub struct AutoDetectResponse {
    pub objects: Vec<DetectedObject>,
    pub model: String,
    pub processing_time_ms: u64,
}

/// Auto-detect objects in an image using available models.
/// Currently returns a placeholder response.
/// To enable real inference, integrate an ONNX runtime or
/// call an external Python/CLI tool from here.
#[tauri::command]
pub async fn auto_detect_objects(
    image_path: String,
    confidence_threshold: Option<f32>,
) -> Result<String, String> {
    let _threshold = confidence_threshold.unwrap_or(0.5);

    // Validate image exists
    if !std::path::Path::new(&image_path).exists() {
        return Err(format!("Image not found: {}", image_path));
    }

    // Placeholder: In a real implementation, this would:
    // 1. Load the ONNX model (cached in a lazy_static)
    // 2. Preprocess the image
    // 3. Run inference
    // 4. Post-process results (NMS, etc.)
    let response = AutoDetectResponse {
        objects: vec![],
        model: "none (placeholder)".to_string(),
        processing_time_ms: 0,
    };

    serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))
}

/// Get the dimensions of an image without fully decoding it.
/// Useful for fast image metadata retrieval.
#[tauri::command]
pub fn get_image_dimensions(image_path: String) -> Result<String, String> {
    match imagesize::size(&image_path) {
        Ok(size) => {
            let result = serde_json::json!({
                "width": size.width,
                "height": size.height,
            });
            Ok(result.to_string())
        }
        Err(e) => Err(format!("Failed to get image dimensions: {:?}", e)),
    }
}

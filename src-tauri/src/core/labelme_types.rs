use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabelMeShape {
    pub label: String,
    pub points: Vec<[f32; 2]>,
    pub shape_type: String,
    pub group_id: Option<serde_json::Value>,
    pub flags: Option<HashMap<String, bool>>,
    // Capture any other fields dynamically
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabelMeFile {
    pub version: Option<String>,
    pub flags: Option<HashMap<String, bool>>,
    pub shapes: Vec<LabelMeShape>,
    #[serde(rename = "imagePath")]
    pub image_path: String,
    #[serde(rename = "imageData")]
    pub image_data: Option<String>,
    #[serde(rename = "imageHeight")]
    pub image_height: u32,
    #[serde(rename = "imageWidth")]
    pub image_width: u32,
    // Capture any other fields dynamically
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// Helper struct for bounding boxes
#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
   pub x_min: f32,
   pub y_min: f32,
   pub x_max: f32,
   pub y_max: f32,
}

impl BoundingBox {
    // Check if this bounding box is fully contained within another
    #[allow(dead_code)]
    pub fn is_inside(&self, other: &BoundingBox) -> bool {
        self.x_min >= other.x_min &&
        self.y_min >= other.y_min &&
        self.x_max <= other.x_max &&
        self.y_max <= other.y_max
    }

    // Check if this bounding box overlaps with another
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        // Check for separation on each axis
        // They overlap if they are *not* separated on any axis
        self.x_min < other.x_max && // self is not completely to the right of other
        self.x_max > other.x_min && // self is not completely to the left of other
        self.y_min < other.y_max && // self is not completely below other
        self.y_max > other.y_min    // self is not completely above other
    }
}

// Helper function to get bounding box from points
pub fn get_bounding_box(points: &[[f32; 2]]) -> Option<BoundingBox> {
    if points.is_empty() {
        return None;
    }
    let mut x_min = points[0][0];
    let mut y_min = points[0][1];
    let mut x_max = points[0][0];
    let mut y_max = points[0][1];

    for point in points.iter().skip(1) {
        x_min = x_min.min(point[0]);
        y_min = y_min.min(point[1]);
        x_max = x_max.max(point[0]);
        y_max = y_max.max(point[1]);
    }
    Some(BoundingBox { x_min, y_min, x_max, y_max })
}
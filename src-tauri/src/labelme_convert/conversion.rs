// LabelMe format conversion utilities
// Based on GreatV/labelme2yolo (MIT License)
// Original: https://github.com/GreatV/labelme2yolo
// Copyright (c) 2024 GreatV
//
// Adapted and modified for dataset-app

use crate::labelme_convert::config::AnnotationFormat;
use crate::labelme_convert::detection::validate_shape_points;
use crate::labelme_convert::types::{InputAnnotationFormat, InvalidReason, Shape};

// Re-export Split, determine_split, hash_string from pipeline for backward compatibility
pub use crate::labelme_convert::pipeline::{determine_split, hash_string, Split};

/// Calculate bounding box from shape points
/// Returns (x_center, y_center, width, height) normalized to [0, 1]
pub fn calculate_bbox(
    shape: &Shape,
    image_width: u32,
    image_height: u32,
) -> Option<(f64, f64, f64, f64)> {
    if shape.points.is_empty() {
        return None;
    }

    let (min_x, max_x, min_y, max_y) = get_bounding_coords(&shape.points);

    // Clamp to image bounds
    let min_x = min_x.max(0.0).min(image_width as f64);
    let max_x = max_x.max(0.0).min(image_width as f64);
    let min_y = min_y.max(0.0).min(image_height as f64);
    let max_y = max_y.max(0.0).min(image_height as f64);

    let width = max_x - min_x;
    let height = max_y - min_y;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let x_center = (min_x + max_x) / 2.0;
    let y_center = (min_y + max_y) / 2.0;

    // Normalize to [0, 1]
    let inv_w = 1.0 / image_width as f64;
    let inv_h = 1.0 / image_height as f64;

    Some((
        x_center * inv_w,
        y_center * inv_h,
        width * inv_w,
        height * inv_h,
    ))
}

/// Get bounding coordinates from points
/// Returns (min_x, max_x, min_y, max_y)
fn get_bounding_coords(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for (x, y) in points {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    (min_x, max_x, min_y, max_y)
}

/// Normalize polygon points to [0, 1] range
pub fn normalize_polygon(
    points: &[(f64, f64)],
    image_width: u32,
    image_height: u32,
) -> Vec<(f64, f64)> {
    let inv_w = 1.0 / image_width as f64;
    let inv_h = 1.0 / image_height as f64;

    points
        .iter()
        .map(|(x, y)| {
            // Clamp and normalize
            let nx = (x.max(0.0).min(image_width as f64)) * inv_w;
            let ny = (y.max(0.0).min(image_height as f64)) * inv_h;
            (nx, ny)
        })
        .collect()
}

/// Convert shape to YOLO format string with strict validation based on detected input format
/// Returns Err with reason if the shape cannot be converted
pub fn shape_to_yolo_line(
    shape: &Shape,
    class_id: usize,
    image_width: u32,
    image_height: u32,
    format: AnnotationFormat,
    input_format: InputAnnotationFormat,
) -> Result<String, InvalidReason> {
    // Use the shared validation function from detection.rs
    validate_shape_points(shape, input_format)?;

    // Now proceed with conversion based on output format
    match format {
        AnnotationFormat::Bbox => {
            // For bbox output, we always calculate bounding box regardless of input format
            let (x_center, y_center, width, height) =
                calculate_bbox(shape, image_width, image_height)
                    .ok_or_else(|| {
                        if shape.points.is_empty() {
                            InvalidReason::EmptyPoints
                        } else {
                            InvalidReason::ZeroArea
                        }
                    })?;

            Ok(format!(
                "{} {:.6} {:.6} {:.6} {:.6}",
                class_id, x_center, y_center, width, height
            ))
        }
        AnnotationFormat::Polygon => {
            // For polygon output
            match input_format {
                InputAnnotationFormat::Bbox2Point => {
                    // Convert 2-point bbox to 4-point polygon
                    let expanded = rectangle_to_polygon(&shape.points);
                    let normalized = normalize_polygon(&expanded, image_width, image_height);
                    let mut line = class_id.to_string();
                    for (x, y) in normalized {
                        line.push_str(&format!(" {:.6} {:.6}", x, y));
                    }
                    Ok(line)
                }
                InputAnnotationFormat::Bbox4Point | InputAnnotationFormat::Polygon | InputAnnotationFormat::Unknown => {
                    // Already has enough points, output as-is
                    let normalized = normalize_polygon(&shape.points, image_width, image_height);
                    let mut line = class_id.to_string();
                    for (x, y) in normalized {
                        line.push_str(&format!(" {:.6} {:.6}", x, y));
                    }
                    Ok(line)
                }
            }
        }
    }
}

/// Convert rectangle (2 points) to polygon (4 points)
pub fn rectangle_to_polygon(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if points.len() != 2 {
        return points.to_vec();
    }

    let (x1, y1) = points[0];
    let (x2, y2) = points[1];

    vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)]
}

/// Convert circle to polygon approximation
pub fn circle_to_polygon(center: (f64, f64), radius: f64, num_points: usize) -> Vec<(f64, f64)> {
    let (cx, cy) = center;
    let mut polygon = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (num_points as f64);
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        polygon.push((x, y));
    }

    polygon
}

/// Calculate polygon area using the shoelace formula
pub fn calculate_polygon_area(points: &[(f64, f64)]) -> f64 {
    if points.len() < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    let n = points.len();

    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].0 * points[j].1;
        area -= points[j].0 * points[i].1;
    }

    (area / 2.0).abs()
}

/// Calculate bounding box from polygon points for COCO format
/// Returns [x, y, width, height] where (x, y) is top-left corner
pub fn calculate_coco_bbox(points: &[(f64, f64)]) -> [f64; 4] {
    if points.is_empty() {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let (min_x, max_x, min_y, max_y) = get_bounding_coords(points);

    [min_x, min_y, max_x - min_x, max_y - min_y]
}

/// Flatten polygon points for COCO segmentation format
/// Converts [(x1, y1), (x2, y2), ...] to [x1, y1, x2, y2, ...]
pub fn flatten_polygon(points: &[(f64, f64)]) -> Vec<f64> {
    points.iter().flat_map(|(x, y)| vec![*x, *y]).collect()
}

// Note: Split, determine_split, hash_string are re-exported from pipeline.rs
// Note: detect_input_format and detect_input_format_from_annotations
// have been moved to detection.rs for better modularity.
// Use crate::labelme_convert::detection::detect_input_format instead.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bbox() {
        let shape = Shape {
            label: "test".to_string(),
            points: vec![(10.0, 10.0), (30.0, 10.0), (30.0, 30.0), (10.0, 30.0)],
            group_id: None,
            shape_type: "polygon".to_string(),
            description: None,
            mask: None,
            flags: None,
        };

        let result = calculate_bbox(&shape, 100, 100).unwrap();
        assert!((result.0 - 0.2).abs() < 0.001); // x_center
        assert!((result.1 - 0.2).abs() < 0.001); // y_center
        assert!((result.2 - 0.2).abs() < 0.001); // width
        assert!((result.3 - 0.2).abs() < 0.001); // height
    }

    #[test]
    fn test_normalize_polygon() {
        let points = vec![(50.0, 50.0), (100.0, 50.0), (100.0, 100.0)];
        let normalized = normalize_polygon(&points, 200, 200);

        assert!((normalized[0].0 - 0.25).abs() < 0.001);
        assert!((normalized[0].1 - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_rectangle_to_polygon() {
        let points = vec![(10.0, 10.0), (30.0, 30.0)];
        let polygon = rectangle_to_polygon(&points);

        assert_eq!(polygon.len(), 4);
        assert_eq!(polygon[0], (10.0, 10.0));
        assert_eq!(polygon[1], (30.0, 10.0));
        assert_eq!(polygon[2], (30.0, 30.0));
        assert_eq!(polygon[3], (10.0, 30.0));
    }

    #[test]
    fn test_polygon_area() {
        // Square 10x10
        let points = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
        let area = calculate_polygon_area(&points);
        assert!((area - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_determine_split() {
        // Test with specific hash values
        assert_eq!(determine_split(100, 0.2, 0.1), Split::Val); // 0.1 < 0.2
        assert_eq!(determine_split(250, 0.2, 0.1), Split::Test); // 0.25 between 0.2 and 0.3
        assert_eq!(determine_split(500, 0.2, 0.1), Split::Train); // 0.5 > 0.3
    }
}

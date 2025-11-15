use opencv::{
    core::{Point, Scalar /* Vector, Point2f */},
    imgcodecs, imgproc,
};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::Path;
// use image::{RgbImage, Rgb};
// use imageproc::drawing::{draw_polygon_mut, draw_filled_rect_mut};

/// Draw polygons on an image based on LabelMe JSON annotation
///
/// This function reads a LabelMe JSON file and the corresponding image,
/// draws polygon shapes for all annotations, and saves the result.
///
/// # Arguments
/// * `source_dir` - Directory containing images
/// * `json_path` - Path to the LabelMe JSON annotation file
/// * `output_dir` - Directory to save the output image
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error
pub fn draw_polygons(
    source_dir: &str,
    json_path: &str,
    output_dir: &str,
) -> Result<(), Box<dyn Error>> {
    // Read and parse the JSON file
    let json_content = fs::read_to_string(json_path)?;
    let json: Value = serde_json::from_str(&json_content)?;

    // Extract image path from the JSON
    let image_path = json["imagePath"]
        .as_str()
        .ok_or("Missing imagePath in JSON")?;
    let image_file = Path::new(source_dir).join(image_path);

    // Read the image
    let img = imgcodecs::imread(image_file.to_str().unwrap(), imgcodecs::IMREAD_COLOR)
        .map_err(|e| format!("Failed to read image: {:?}", e))?;

    // Create a copy for drawing
    let mut output_img = img.clone();

    // Process each shape in the JSON
    if let Some(shapes) = json["shapes"].as_array() {
        for shape in shapes {
            let label = shape["label"].as_str().unwrap_or("unknown");
            let shape_type = shape["shape_type"].as_str().unwrap_or("");

            // Process polygon shapes
            if shape_type == "polygon" {
                if let Some(points) = shape["points"].as_array() {
                    // Create vectors of points for the polygon
                    let mut contour_points = Vec::new();
                    for point in points {
                        if let (Some(x), Some(y)) = (point[0].as_f64(), point[1].as_f64()) {
                            contour_points.push(Point::new(x as i32, y as i32));
                        }
                    }

                    // Draw the polygon lines manually
                    if contour_points.len() >= 2 {
                        for i in 0..contour_points.len() {
                            let start_point = contour_points[i];
                            let end_point = contour_points[(i + 1) % contour_points.len()];

                            imgproc::line(
                                &mut output_img,
                                start_point,
                                end_point,
                                Scalar::new(0.0, 255.0, 255.0, 0.0), // Yellow color
                                2,                                   // Line thickness
                                imgproc::LINE_8,
                                0,
                            )?;
                        }

                        // Add label text if we have points
                        if !contour_points.is_empty() {
                            let text_point = contour_points[0];
                            imgproc::put_text(
                                &mut output_img,
                                label,
                                Point::new(text_point.x, text_point.y - 10),
                                imgproc::FONT_HERSHEY_SIMPLEX,
                                0.5,
                                Scalar::new(0.0, 255.0, 255.0, 0.0), // Yellow color
                                1,
                                imgproc::LINE_8,
                                false,
                            )?;
                        }
                    }
                }
            }
            // Process rectangle/bounding_box shapes
            else if shape_type == "rectangle" || shape_type == "bounding_box" {
                if let Some(points) = shape["points"].as_array() {
                    if points.len() >= 2 {
                        let x1 = points[0][0].as_f64().unwrap_or(0.0) as i32;
                        let y1 = points[0][1].as_f64().unwrap_or(0.0) as i32;
                        let x2 = points[1][0].as_f64().unwrap_or(0.0) as i32;
                        let y2 = points[1][1].as_f64().unwrap_or(0.0) as i32;

                        // Draw rectangle directly instead of using contours
                        imgproc::rectangle(
                            &mut output_img,
                            opencv::core::Rect::new(x1, y1, x2 - x1, y2 - y1),
                            Scalar::new(255.0, 0.0, 0.0, 0.0), // Red color
                            2,                                 // Line thickness
                            imgproc::LINE_8,
                            0,
                        )?;

                        // Add label text
                        imgproc::put_text(
                            &mut output_img,
                            label,
                            Point::new(x1, y1 - 10),
                            imgproc::FONT_HERSHEY_SIMPLEX,
                            0.5,
                            Scalar::new(255.0, 0.0, 0.0, 0.0), // Red color
                            1,
                            imgproc::LINE_8,
                            false,
                        )?;
                    }
                }
            }
        }
    }

    // Create output filename
    let json_filename = Path::new(json_path).file_name().unwrap().to_str().unwrap();
    let output_filename = format!("{}_polygons.jpg", json_filename.replace(".json", ""));
    let output_path = Path::new(output_dir).join(output_filename);

    // Save the image
    imgcodecs::imwrite(
        output_path.to_str().unwrap(),
        &output_img,
        &opencv::core::Vector::new(),
    )?;

    Ok(())
}

// LabelMe dataset format detection
// Provides intelligent detection of input annotation formats

use crate::labelme_convert::io::{find_json_files, read_labelme_json};
use crate::labelme_convert::types::{InputAnnotationFormat, Shape};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

/// Detailed analysis result of a dataset
#[derive(Debug, Clone, Serialize)]
pub struct DatasetAnalysis {
    /// Detected input format
    pub input_format: InputAnnotationFormat,
    /// Total number of JSON files in the dataset
    pub total_files: usize,
    /// Number of files sampled for analysis
    pub sample_files: usize,
    /// Total number of annotations sampled
    pub sample_annotations: usize,
    /// Confidence score (0.0 - 1.0) for the detected format
    pub confidence: f64,
    /// Distribution of point counts in sampled annotations
    pub points_distribution: HashMap<usize, usize>,
    /// Human-readable description of the detected format
    pub format_description: String,
}

impl Default for DatasetAnalysis {
    fn default() -> Self {
        Self {
            input_format: InputAnnotationFormat::Unknown,
            total_files: 0,
            sample_files: 0,
            sample_annotations: 0,
            confidence: 0.0,
            points_distribution: HashMap::new(),
            format_description: "未知格式".to_string(),
        }
    }
}

/// Configuration for dataset analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Maximum number of files to sample
    pub max_sample_files: usize,
    /// Maximum number of annotations to analyze
    pub max_sample_annotations: usize,
    /// Confidence threshold for format detection (0.0 - 1.0)
    pub confidence_threshold: f64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_sample_files: 20,
            max_sample_annotations: 100,
            confidence_threshold: 0.8,
        }
    }
}

/// Analyze a dataset directory to detect the input annotation format
///
/// # Arguments
/// * `input_dir` - Path to the directory containing LabelMe JSON files
///
/// # Returns
/// A `DatasetAnalysis` containing the detected format and confidence metrics
pub fn analyze_dataset(input_dir: &Path) -> DatasetAnalysis {
    analyze_dataset_with_config(input_dir, &AnalysisConfig::default())
}

/// Analyze a dataset directory with custom configuration
pub fn analyze_dataset_with_config(input_dir: &Path, config: &AnalysisConfig) -> DatasetAnalysis {
    let json_files = find_json_files(input_dir);

    if json_files.is_empty() {
        return DatasetAnalysis {
            format_description: "找不到 JSON 檔案".to_string(),
            ..Default::default()
        };
    }

    let total_files = json_files.len();
    let sample_files = total_files.min(config.max_sample_files);

    // Collect shapes from sampled files
    let mut all_shapes: Vec<Shape> = Vec::new();

    for json_path in json_files.iter().take(sample_files) {
        if let Ok(annotation) = read_labelme_json(json_path) {
            all_shapes.extend(annotation.shapes);

            // Stop if we have enough annotations
            if all_shapes.len() >= config.max_sample_annotations {
                break;
            }
        }
    }

    let shape_refs: Vec<&Shape> = all_shapes.iter().collect();
    analyze_shapes(&shape_refs, total_files, sample_files, config)
}

/// Analyze a collection of shapes to determine the input format
pub fn analyze_shapes(
    shapes: &[&Shape],
    total_files: usize,
    sample_files: usize,
    config: &AnalysisConfig,
) -> DatasetAnalysis {
    if shapes.is_empty() {
        return DatasetAnalysis {
            total_files,
            sample_files,
            format_description: "沒有找到任何標註".to_string(),
            ..Default::default()
        };
    }

    // Count points distribution
    let mut points_distribution: HashMap<usize, usize> = HashMap::new();
    let sample_annotations = shapes.len().min(config.max_sample_annotations);

    for shape in shapes.iter().take(sample_annotations) {
        *points_distribution.entry(shape.points.len()).or_insert(0) += 1;
    }

    // Analyze the distribution
    let count_2_points = *points_distribution.get(&2).unwrap_or(&0);
    let count_4_points = *points_distribution.get(&4).unwrap_or(&0);
    let count_3_plus_points: usize = points_distribution
        .iter()
        .filter(|(&k, _)| k >= 3)
        .map(|(_, &v)| v)
        .sum();

    // Calculate confidence and determine format
    let (input_format, confidence, format_description) = determine_format_with_confidence(
        sample_annotations,
        count_2_points,
        count_4_points,
        count_3_plus_points,
        &points_distribution,
        config.confidence_threshold,
    );

    DatasetAnalysis {
        input_format,
        total_files,
        sample_files,
        sample_annotations,
        confidence,
        points_distribution,
        format_description,
    }
}

/// Determine the format with confidence calculation
fn determine_format_with_confidence(
    total_sampled: usize,
    count_2_points: usize,
    count_4_points: usize,
    count_3_plus_points: usize,
    points_distribution: &HashMap<usize, usize>,
    threshold: f64,
) -> (InputAnnotationFormat, f64, String) {
    if total_sampled == 0 {
        return (
            InputAnnotationFormat::Unknown,
            0.0,
            "沒有標註可供分析".to_string(),
        );
    }

    let ratio_2_points = count_2_points as f64 / total_sampled as f64;
    let ratio_4_points = count_4_points as f64 / total_sampled as f64;
    let ratio_3_plus_points = count_3_plus_points as f64 / total_sampled as f64;

    // Check for 2-point bbox format (highest priority for exact match)
    if ratio_2_points >= threshold {
        return (
            InputAnnotationFormat::Bbox2Point,
            ratio_2_points,
            format!(
                "2 點邊界框（對角線表示法），{:.1}% 的標註符合",
                ratio_2_points * 100.0
            ),
        );
    }

    // Check for 4-point bbox format
    if ratio_4_points >= threshold {
        return (
            InputAnnotationFormat::Bbox4Point,
            ratio_4_points,
            format!(
                "4 點邊界框（四角落表示法），{:.1}% 的標註符合",
                ratio_4_points * 100.0
            ),
        );
    }

    // Check for polygon format (3+ points with variation)
    if ratio_3_plus_points >= threshold {
        // Distinguish between 4-point bbox and polygon
        if ratio_4_points >= threshold {
            return (
                InputAnnotationFormat::Bbox4Point,
                ratio_4_points,
                format!(
                    "4 點邊界框（四角落表示法），{:.1}% 的標註符合",
                    ratio_4_points * 100.0
                ),
            );
        }

        // Check if points vary (true polygon) or are all the same count
        let unique_point_counts: Vec<_> = points_distribution
            .iter()
            .filter(|(&k, &v)| k >= 3 && v > 0)
            .collect();

        if unique_point_counts.len() > 1 {
            // Variable point counts = polygon
            return (
                InputAnnotationFormat::Polygon,
                ratio_3_plus_points,
                format!(
                    "多邊形標註（點數可變），{:.1}% 的標註有 3 點以上",
                    ratio_3_plus_points * 100.0
                ),
            );
        } else if ratio_4_points > 0.5 {
            // Mostly 4 points = likely bbox
            return (
                InputAnnotationFormat::Bbox4Point,
                ratio_4_points,
                format!(
                    "4 點邊界框（四角落表示法），{:.1}% 的標註符合",
                    ratio_4_points * 100.0
                ),
            );
        } else {
            return (
                InputAnnotationFormat::Polygon,
                ratio_3_plus_points,
                format!(
                    "多邊形標註，{:.1}% 的標註有 3 點以上",
                    ratio_3_plus_points * 100.0
                ),
            );
        }
    }

    // Mixed or unknown format
    let max_ratio = ratio_2_points.max(ratio_4_points).max(ratio_3_plus_points);
    (
        InputAnnotationFormat::Unknown,
        max_ratio,
        format!(
            "混合格式：2 點 {:.1}%、4 點 {:.1}%、3+ 點 {:.1}%",
            ratio_2_points * 100.0,
            ratio_4_points * 100.0,
            ratio_3_plus_points * 100.0
        ),
    )
}

/// Quick format detection from shapes (for use during conversion)
/// This is a simplified version that just returns the format without detailed analysis
pub fn detect_input_format(shapes: &[&Shape]) -> InputAnnotationFormat {
    if shapes.is_empty() {
        return InputAnnotationFormat::Unknown;
    }

    let config = AnalysisConfig::default();
    let analysis = analyze_shapes(shapes, 0, 0, &config);
    analysis.input_format
}

/// Detect input format from a list of LabelMe annotations
pub fn detect_input_format_from_annotations(
    annotations: &[crate::labelme_convert::types::LabelMeAnnotation],
) -> InputAnnotationFormat {
    let shapes: Vec<&Shape> = annotations
        .iter()
        .flat_map(|a| a.shapes.iter())
        .collect();

    detect_input_format(&shapes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_shape(points_count: usize) -> Shape {
        Shape {
            label: "test".to_string(),
            points: (0..points_count).map(|i| (i as f64, i as f64)).collect(),
            group_id: None,
            shape_type: "polygon".to_string(),
            description: None,
            mask: None,
            flags: None,
        }
    }

    #[test]
    fn test_detect_2point_bbox() {
        let shapes: Vec<Shape> = (0..10).map(|_| create_test_shape(2)).collect();
        let shape_refs: Vec<&Shape> = shapes.iter().collect();

        let analysis = analyze_shapes(&shape_refs, 10, 10, &AnalysisConfig::default());

        assert_eq!(analysis.input_format, InputAnnotationFormat::Bbox2Point);
        assert!(analysis.confidence >= 0.8);
    }

    #[test]
    fn test_detect_4point_bbox() {
        let shapes: Vec<Shape> = (0..10).map(|_| create_test_shape(4)).collect();
        let shape_refs: Vec<&Shape> = shapes.iter().collect();

        let analysis = analyze_shapes(&shape_refs, 10, 10, &AnalysisConfig::default());

        assert_eq!(analysis.input_format, InputAnnotationFormat::Bbox4Point);
        assert!(analysis.confidence >= 0.8);
    }

    #[test]
    fn test_detect_polygon() {
        // Mix of different point counts (all >= 3)
        let shapes: Vec<Shape> = vec![
            create_test_shape(5),
            create_test_shape(6),
            create_test_shape(7),
            create_test_shape(8),
            create_test_shape(5),
            create_test_shape(6),
            create_test_shape(9),
            create_test_shape(10),
            create_test_shape(5),
            create_test_shape(7),
        ];
        let shape_refs: Vec<&Shape> = shapes.iter().collect();

        let analysis = analyze_shapes(&shape_refs, 10, 10, &AnalysisConfig::default());

        assert_eq!(analysis.input_format, InputAnnotationFormat::Polygon);
    }

    #[test]
    fn test_detect_mixed_format() {
        // 30% 2-point, 30% 4-point, 40% other
        let mut shapes: Vec<Shape> = Vec::new();
        shapes.extend((0..3).map(|_| create_test_shape(2)));
        shapes.extend((0..3).map(|_| create_test_shape(4)));
        shapes.extend((0..4).map(|_| create_test_shape(6)));

        let shape_refs: Vec<&Shape> = shapes.iter().collect();

        let analysis = analyze_shapes(&shape_refs, 10, 10, &AnalysisConfig::default());

        // Should detect as polygon since 3+ points is the majority
        assert!(analysis.confidence < 0.8 || analysis.input_format == InputAnnotationFormat::Polygon);
    }

    #[test]
    fn test_empty_shapes() {
        let shapes: Vec<&Shape> = vec![];
        let analysis = analyze_shapes(&shapes, 0, 0, &AnalysisConfig::default());

        assert_eq!(analysis.input_format, InputAnnotationFormat::Unknown);
        assert_eq!(analysis.confidence, 0.0);
    }
}

#![allow(dead_code)]

use anyhow::Result;
use opencv::{core::Vector, imgcodecs, prelude::*, videoio};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Single,
    Separate,
}

#[derive(Debug, Deserialize)]
pub struct FileFormat {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Deserialize)]
pub struct ExtractionConfig {
    pub frame_digits: usize,
    pub skip_frames: u32,
    pub filename_separator: String,
    pub output_mode: OutputMode,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub input_dir: String,
    pub output_dir: String,
    pub file_format: FileFormat,
    pub extraction: ExtractionConfig,
}

fn get_video_name(video_path: &Path) -> String {
    video_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn get_output_path(base_output_dir: &Path, video_name: &str, config: &Config) -> PathBuf {
    match config.extraction.output_mode {
        OutputMode::Separate => base_output_dir.join(video_name),
        OutputMode::Single => base_output_dir.to_path_buf(),
    }
}

pub fn process_single_video(
    video_path: &Path,
    output_dir: &Path,
    config: &Config,
    params: &Vector<i32>,
) -> Result<u32> {
    let video_name = get_video_name(video_path);
    let mut frame_count = 0;
    let mut frame_skip_counter = 0;

    // Get the appropriate output directory based on mode
    let video_output_path = get_output_path(output_dir, &video_name, config);
    fs::create_dir_all(&video_output_path)?;

    // Open video file
    let mut cap = videoio::VideoCapture::from_file(video_path.to_str().unwrap(), videoio::CAP_ANY)?;

    if !cap.is_opened()? {
        println!("Error: Could not open video {:?}", video_path);
        return Ok(0);
    }

    // Get total frames (safely convert to i32)
    let total_frames = match cap.get(videoio::CAP_PROP_FRAME_COUNT) {
        Ok(frames) => frames as i32,
        Err(_) => -1, // Unknown frame count
    };

    if total_frames > 0 {
        println!(
            "Processing video: {:?} (Total frames: {})",
            video_path.file_name().unwrap(),
            total_frames
        );
    } else {
        println!(
            "Processing video: {:?} (Frame count unknown)",
            video_path.file_name().unwrap()
        );
    }

    let mut frame = Mat::default();
    while cap.read(&mut frame)? {
        // Handle frame skipping
        if frame_skip_counter < config.extraction.skip_frames {
            frame_skip_counter += 1;
            continue;
        }
        frame_skip_counter = 0;

        // Save frame with video name as prefix
        let frame_filename = video_output_path.join(format!(
            "{}{}{:0width$}.{}",
            video_name,
            config.extraction.filename_separator,
            frame_count,
            config.file_format.output,
            width = config.extraction.frame_digits
        ));

        imgcodecs::imwrite(frame_filename.to_str().unwrap(), &frame, params)?;
        frame_count += 1;

        // Print progress every 100 frames
        if frame_count % 100 == 0 {
            if total_frames > 0 {
                let progress = (frame_count as f32 / total_frames as f32 * 100.0) as i32;
                println!(
                    "  Progress: {} frames extracted from {} ({}%)",
                    frame_count, video_name, progress
                );
            } else {
                println!(
                    "  Progress: {} frames extracted from {}",
                    frame_count, video_name
                );
            }
        }
    }

    println!(
        "Completed {:?} - Extracted {} frames",
        video_path.file_name().unwrap(),
        frame_count
    );
    Ok(frame_count)
}

pub fn extract_frames_from_videos(config: &Config) -> Result<()> {
    let input_path = Path::new(&config.input_dir);
    let output_path = Path::new(&config.output_dir);

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)?;

    // Get all video files with configured extension
    let video_files: Vec<PathBuf> = fs::read_dir(input_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case(&config.file_format.input))
                .unwrap_or(false)
        })
        .collect();

    if video_files.is_empty() {
        println!(
            "No {} files found in {}",
            config.file_format.input,
            input_path.display()
        );
        return Ok(());
    }

    println!("Found {} video files to process", video_files.len());
    println!("Output mode: {:?}", config.extraction.output_mode);

    // Prepare image writing parameters for maximum quality
    let mut params = Vector::new();
    params.push(imgcodecs::IMWRITE_JPEG_QUALITY);
    params.push(100); // Always use maximum quality (100)

    let mut total_frame_count = 0;

    for video_file in &video_files {
        match process_single_video(video_file, output_path, config, &params) {
            Ok(frames) => total_frame_count += frames,
            Err(e) => println!("Error processing {:?}: {}", video_file, e),
        }
    }

    println!("\nFrame extraction completed!");
    println!("Total videos processed: {}", video_files.len());
    println!("Total frames extracted: {}", total_frame_count);
    Ok(())
}

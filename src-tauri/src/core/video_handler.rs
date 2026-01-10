use anyhow::Result;
use opencv::{
    prelude::*,
    videoio::{
        VideoCapture, VideoCaptureAPIs::CAP_ANY, CAP_PROP_FOURCC, CAP_PROP_FPS,
        CAP_PROP_FRAME_COUNT, CAP_PROP_FRAME_HEIGHT, CAP_PROP_FRAME_WIDTH,
    },
};
use serde_json::json;

pub struct VideoHandler;

impl VideoHandler {
    pub fn get_video_info(filename: &str) -> Result<String, String> {
        let cap = VideoCapture::from_file(filename, CAP_ANY.into()).map_err(|e| e.to_string())?;

        let fps = cap.get(CAP_PROP_FPS).map_err(|e| e.to_string())? as f64;
        let frame_count = cap.get(CAP_PROP_FRAME_COUNT).map_err(|e| e.to_string())? as f64;
        let duration_seconds = frame_count / fps;

        let codec = cap.get(CAP_PROP_FOURCC).map_err(|e| e.to_string())? as i32;
        let codec_str = format!(
            "{}{}{}{}",
            (codec & 0xFF) as u8 as char,
            ((codec >> 8) & 0xFF) as u8 as char,
            ((codec >> 16) & 0xFF) as u8 as char,
            ((codec >> 24) & 0xFF) as u8 as char
        );

        let codec_name = match codec_str.as_str() {
            "avc1" | "h264" => "H264",
            "hev1" | "hvc1" => "H265",
            "mp4v" => "MPEG-4 Part 2",
            "mp4a" => "MPEG-4 AAC",
            _ => "Unknown",
        }
        .to_string();

        let width = cap.get(CAP_PROP_FRAME_WIDTH).map_err(|e| e.to_string())? as i32;
        let height = cap.get(CAP_PROP_FRAME_HEIGHT).map_err(|e| e.to_string())? as i32;

        let info = json!({
            "duration_seconds": duration_seconds,
            "codec_name": codec_name,
            "codec_str": codec_str,
            "resolution": format!("{}x{}", width, height),
            "frame_count": frame_count,
            "fps": fps
        });

        Ok(info.to_string())
    }

    pub fn read_video_file(file_path: &str) -> Result<String, String> {
        match VideoCapture::from_file(file_path, CAP_ANY.into()) {
            Ok(cap) => {
                let frame_count = cap.get(CAP_PROP_FRAME_COUNT).unwrap_or(0.0) as i32;
                let fps = cap.get(CAP_PROP_FPS).unwrap_or(0.0);
                let codec = cap.get(CAP_PROP_FOURCC).unwrap_or(0.0) as i32;
                let width = cap.get(CAP_PROP_FRAME_WIDTH).unwrap_or(0.0) as i32;
                let height = cap.get(CAP_PROP_FRAME_HEIGHT).unwrap_or(0.0) as i32;
                let duration = if fps > 0.0 {
                    frame_count as f64 / fps
                } else {
                    0.0
                };

                let codec_str = format!(
                    "{}{}{}{}",
                    (codec & 0xFF) as u8 as char,
                    ((codec >> 8) & 0xFF) as u8 as char,
                    ((codec >> 16) & 0xFF) as u8 as char,
                    ((codec >> 24) & 0xFF) as u8 as char
                );

                let info = format!(
                    "Video details:\nDuration: {:.2} seconds\nTotal Frames: {}
FPS: {:.2}\nCodec: {}
Resolution: {}x{}",
                    duration, frame_count, fps, codec_str, width, height
                );
                println!("{}", info);
                Ok(info)
            }
            Err(e) => {
                println!("Failed to open video: {}", e);
                Err("Failed to open video".into())
            }
        }
    }
}

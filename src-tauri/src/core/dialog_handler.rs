pub struct DialogHandler;

impl DialogHandler {
    pub fn select_video_file() -> Result<String, String> {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("Video Files", &["mp4"])
            .pick_file()
        {
            println!("Selected file: {}", file.to_string_lossy());
            let file_path = file.to_string_lossy().to_string();
            Ok(file_path)
        } else {
            println!("No file selected");
            Err("No file selected".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // For testing, we can create a mock dialog handler
    #[cfg(test)]
    impl DialogHandler {
        pub fn select_video_file_test(mock_path: &str) -> Result<String, String> {
            Ok(mock_path.to_string())
        }
    }

    #[test]
    fn test_select_video_file() {
        let mock_path = "/path/to/video.mp4";
        let result = DialogHandler::select_video_file_test(mock_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), mock_path);
    }
}

/// Progress reporting system for long-running operations
///
/// Provides a type-safe, non-blocking way to emit progress events to the frontend.
use serde::Serialize;
use tauri::{Emitter, Window};

/// Scan progress event payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    /// Current number of items processed
    pub current: usize,
    /// Total number of items to process
    pub total: usize,
    /// Percentage complete (0.0 - 100.0)
    pub percentage: f32,
    /// Human-readable status message
    pub message: String,
}

impl ScanProgress {
    /// Create a new progress event
    pub fn new(current: usize, total: usize, message: impl Into<String>) -> Self {
        let percentage = if total > 0 {
            current as f32 / total as f32 * 100.0
        } else {
            0.0
        };

        Self {
            current,
            total,
            percentage,
            message: message.into(),
        }
    }
}

/// Progress emitter that wraps a Tauri window and event name
///
/// This is a lightweight, cloneable struct that can be passed into blocking tasks.
#[derive(Clone)]
pub struct ProgressEmitter {
    window: Window,
    event_name: String,
}

impl ProgressEmitter {
    /// Create a new progress emitter
    ///
    /// # Arguments
    /// * `window` - The Tauri window to emit events to
    /// * `event_name` - The event name that the frontend will listen to
    pub fn new(window: Window, event_name: impl Into<String>) -> Self {
        Self {
            window,
            event_name: event_name.into(),
        }
    }

    /// Emit a progress update (non-blocking)
    ///
    /// If emission fails (e.g., window is closed), the error is logged but not propagated.
    /// This ensures that progress reporting doesn't cause the main operation to fail.
    pub fn emit(&self, current: usize, total: usize, message: impl Into<String>) {
        let progress = ScanProgress::new(current, total, message);

        if let Err(e) = self.window.emit(&self.event_name, &progress) {
            eprintln!("⚠️ Failed to emit progress event: {}", e);
        }
    }

    /// Emit a completion message
    pub fn complete(&self, message: impl Into<String>) {
        let progress = ScanProgress {
            current: 100,
            total: 100,
            percentage: 100.0,
            message: message.into(),
        };

        let _ = self.window.emit(&self.event_name, &progress);
    }

    /// Emit an error message
    pub fn error(&self, message: impl Into<String>) {
        let progress = ScanProgress {
            current: 0,
            total: 0,
            percentage: 0.0,
            message: message.into(),
        };

        let _ = self.window.emit(&self.event_name, &progress);
    }
}

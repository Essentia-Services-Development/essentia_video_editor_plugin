//! Video editor configuration.

use crate::types::{FrameRate, Resolution};

/// Configuration for the video editor plugin.
#[derive(Debug, Clone)]
pub struct VideoEditorConfig {
    /// Maximum number of tracks.
    pub max_tracks:         usize,
    /// Project resolution.
    pub resolution:         Resolution,
    /// Project frame rate.
    pub frame_rate:         FrameRate,
    /// Enable GPU acceleration.
    pub gpu_acceleration:   bool,
    /// Preview quality (0.0 - 1.0).
    pub preview_quality:    f32,
    /// Auto-save interval (seconds, 0 = disabled).
    pub auto_save_interval: u64,
}

impl Default for VideoEditorConfig {
    fn default() -> Self {
        Self {
            max_tracks:         32,
            resolution:         Resolution::FHD,
            frame_rate:         FrameRate::FPS_30,
            gpu_acceleration:   true,
            preview_quality:    0.5,
            auto_save_interval: 60,
        }
    }
}

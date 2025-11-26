//! Asset library management.

use crate::errors::{VideoEditorError, VideoEditorResult};
use crate::types::{AudioClip, VideoClip};

/// Asset library for managing media files.
pub struct AssetLibrary {
    video_clips: Vec<VideoClip>,
    audio_clips: Vec<AudioClip>,
    next_clip_id: u64,
}

impl AssetLibrary {
    /// Create a new asset library.
    pub fn new() -> Self {
        Self {
            video_clips: Vec::new(),
            audio_clips: Vec::new(),
            next_clip_id: 1,
        }
    }

    /// Import a video file.
    pub fn import_video(&mut self, path: &str) -> VideoEditorResult<u64> {
        if path.is_empty() {
            return Err(VideoEditorError::Asset("Path cannot be empty".into()));
        }

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        // Placeholder - would analyze video file
        self.video_clips.push(VideoClip {
            id,
            source_path: path.to_string(),
            duration_ms: 10000, // Placeholder
            in_point_ms: 0,
            out_point_ms: 10000,
            format: crate::types::VideoFormat::H264,
            resolution: crate::types::Resolution::FHD,
            frame_rate: crate::types::FrameRate::FPS_30,
        });

        Ok(id)
    }

    /// Import an audio file.
    pub fn import_audio(&mut self, path: &str) -> VideoEditorResult<u64> {
        if path.is_empty() {
            return Err(VideoEditorError::Asset("Path cannot be empty".into()));
        }

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        // Placeholder - would analyze audio file
        self.audio_clips.push(AudioClip {
            id,
            source_path: path.to_string(),
            duration_ms: 10000,
            format: crate::types::AudioFormat::AAC,
            sample_rate: 48000,
            channels: 2,
        });

        Ok(id)
    }

    /// Get all video clips.
    pub fn video_clips(&self) -> &[VideoClip] {
        &self.video_clips
    }

    /// Get all audio clips.
    pub fn audio_clips(&self) -> &[AudioClip] {
        &self.audio_clips
    }
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

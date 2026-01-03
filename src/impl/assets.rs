//! Asset library management.

use crate::{
    errors::{VideoEditorError, VideoEditorResult},
    types::{
        AudioClip, AudioFormat, FrameRate, Resolution, TimelinePosition, VideoClip, VideoFormat,
    },
};

/// Asset library for managing media files.
pub struct AssetLibrary {
    video_clips:  Vec<VideoClip>,
    audio_clips:  Vec<AudioClip>,
    next_clip_id: u64,
}

impl AssetLibrary {
    /// Create a new asset library.
    pub fn new() -> Self {
        Self { video_clips: Vec::new(), audio_clips: Vec::new(), next_clip_id: 1 }
    }

    /// Import a video file.
    pub fn import_video(&mut self, path: &str) -> VideoEditorResult<u64> {
        if path.is_empty() {
            return Err(VideoEditorError::Asset("Path cannot be empty".to_string()));
        }

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        // Placeholder - would analyze video file
        let clip = VideoClip::new(id, path)
            .with_resolution(Resolution::FHD)
            .with_frame_rate(FrameRate::FPS_30)
            .with_duration(TimelinePosition::from_ms(10000))
            .with_format(VideoFormat::H264);

        self.video_clips.push(clip);

        Ok(id)
    }

    /// Import an audio file.
    pub fn import_audio(&mut self, path: &str) -> VideoEditorResult<u64> {
        if path.is_empty() {
            return Err(VideoEditorError::Asset("Path cannot be empty".to_string()));
        }

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        // Placeholder - would analyze audio file
        let clip = AudioClip::new(id, path)
            .with_sample_rate(48000)
            .with_channels(2)
            .with_duration(TimelinePosition::from_ms(10000))
            .with_format(AudioFormat::AAC);

        self.audio_clips.push(clip);

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

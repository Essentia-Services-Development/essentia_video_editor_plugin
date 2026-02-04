//! Clip types for media assets.
//!
//! Video and audio clip representations with metadata.

use super::core::{AudioFormat, FrameRate, Resolution, TimePosition, VideoFormat};

/// Video clip state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClipState {
    /// Clip is unloaded.
    #[default]
    Unloaded,
    /// Clip is loading.
    Loading,
    /// Clip is ready for playback.
    Ready,
    /// Clip has an error.
    Error,
    /// Clip is being processed.
    Processing,
}

/// Common clip metadata.
#[derive(Debug, Clone, Default)]
pub struct ClipMetadata {
    /// Clip title.
    pub title:       String,
    /// Clip description.
    pub description: String,
    /// Creation timestamp.
    pub created:     Option<u64>,
    /// Modification timestamp.
    pub modified:    Option<u64>,
    /// Tags for organization.
    pub tags:        Vec<String>,
    /// Custom metadata fields.
    pub custom:      Vec<(String, String)>,
}

impl ClipMetadata {
    /// Creates empty metadata.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Sets the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Adds a tag.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Adds a custom metadata field.
    pub fn add_custom(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom.push((key.into(), value.into()));
    }
}

/// Video clip representation.
#[derive(Debug, Clone)]
pub struct VideoClip {
    /// Unique clip ID.
    pub id:          u64,
    /// File path or URI.
    pub path:        String,
    /// Video resolution.
    pub resolution:  Resolution,
    /// Frame rate.
    pub frame_rate:  FrameRate,
    /// Total duration.
    pub duration:    TimePosition,
    /// Video codec format.
    pub format:      VideoFormat,
    /// Clip state.
    pub state:       ClipState,
    /// Clip metadata.
    pub metadata:    ClipMetadata,
    /// Has audio track.
    pub has_audio:   bool,
    /// Number of frames.
    pub frame_count: u64,
}

impl VideoClip {
    /// Creates a new video clip.
    #[must_use]
    pub fn new(id: u64, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            resolution: Resolution::default(),
            frame_rate: FrameRate::default(),
            duration: TimePosition::default(),
            format: VideoFormat::default(),
            state: ClipState::Unloaded,
            metadata: ClipMetadata::default(),
            has_audio: false,
            frame_count: 0,
        }
    }

    /// Sets the resolution.
    #[must_use]
    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = resolution;
        self
    }

    /// Sets the frame rate.
    #[must_use]
    pub fn with_frame_rate(mut self, frame_rate: FrameRate) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Sets the duration.
    #[must_use]
    pub fn with_duration(mut self, duration: TimePosition) -> Self {
        self.duration = duration;
        self.frame_count = duration.to_frame(&self.frame_rate);
        self
    }

    /// Sets the format.
    #[must_use]
    pub fn with_format(mut self, format: VideoFormat) -> Self {
        self.format = format;
        self
    }

    /// Returns whether the clip is ready.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.state, ClipState::Ready)
    }

    /// Returns the frame at a specific time.
    #[must_use]
    pub fn frame_at(&self, position: TimePosition) -> u64 {
        position.to_frame(&self.frame_rate)
    }
}

/// Audio clip representation.
#[derive(Debug, Clone)]
pub struct AudioClip {
    /// Unique clip ID.
    pub id:           u64,
    /// File path or URI.
    pub path:         String,
    /// Sample rate in Hz.
    pub sample_rate:  u32,
    /// Number of channels.
    pub channels:     u8,
    /// Total duration.
    pub duration:     TimePosition,
    /// Audio codec format.
    pub format:       AudioFormat,
    /// Clip state.
    pub state:        ClipState,
    /// Clip metadata.
    pub metadata:     ClipMetadata,
    /// Bit depth.
    pub bit_depth:    u8,
    /// Total sample count.
    pub sample_count: u64,
}

impl AudioClip {
    /// Creates a new audio clip.
    #[must_use]
    pub fn new(id: u64, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            sample_rate: 48000,
            channels: 2,
            duration: TimePosition::default(),
            format: AudioFormat::default(),
            state: ClipState::Unloaded,
            metadata: ClipMetadata::default(),
            bit_depth: 16,
            sample_count: 0,
        }
    }

    /// Sets the sample rate.
    #[must_use]
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Sets the number of channels.
    #[must_use]
    pub fn with_channels(mut self, channels: u8) -> Self {
        self.channels = channels;
        self
    }

    /// Sets the duration.
    #[must_use]
    pub fn with_duration(mut self, duration: TimePosition) -> Self {
        self.duration = duration;
        self.sample_count = (duration.ms * self.sample_rate as u64) / 1000;
        self
    }

    /// Sets the format.
    #[must_use]
    pub fn with_format(mut self, format: AudioFormat) -> Self {
        self.format = format;
        self
    }

    /// Returns whether the clip is ready.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.state, ClipState::Ready)
    }

    /// Returns the sample at a specific time.
    #[must_use]
    pub fn sample_at(&self, position: TimePosition) -> u64 {
        (position.ms * self.sample_rate as u64) / 1000
    }
}

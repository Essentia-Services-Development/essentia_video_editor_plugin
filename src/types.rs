//! Video editor type definitions.

/// Video clip representation.
#[derive(Debug, Clone)]
pub struct VideoClip {
    /// Clip identifier.
    pub id: u64,
    /// Source file path.
    pub source_path: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Start position in source (trimming).
    pub in_point_ms: u64,
    /// End position in source (trimming).
    pub out_point_ms: u64,
    /// Video format.
    pub format: VideoFormat,
    /// Resolution.
    pub resolution: Resolution,
    /// Frame rate.
    pub frame_rate: FrameRate,
}

/// Audio clip representation.
#[derive(Debug, Clone)]
pub struct AudioClip {
    /// Clip identifier.
    pub id: u64,
    /// Source file path.
    pub source_path: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Audio format.
    pub format: AudioFormat,
    /// Sample rate.
    pub sample_rate: u32,
    /// Channels.
    pub channels: u8,
}

/// Timeline track.
#[derive(Debug, Clone)]
pub struct TimelineTrack {
    /// Track identifier.
    pub id: u64,
    /// Track name.
    pub name: String,
    /// Track type.
    pub track_type: TrackType,
    /// Is track muted.
    pub muted: bool,
    /// Is track locked.
    pub locked: bool,
    /// Track clips.
    pub clips: Vec<TimelineClip>,
}

/// Track type.
#[derive(Debug, Clone, Copy)]
pub enum TrackType {
    Video,
    Audio,
    Text,
    Effect,
}

/// Clip on timeline.
#[derive(Debug, Clone)]
pub struct TimelineClip {
    /// Reference to source clip ID.
    pub clip_id: u64,
    /// Position on timeline (ms).
    pub position: TimelinePosition,
    /// Duration on timeline (ms).
    pub duration_ms: u64,
}

/// Position on timeline.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimelinePosition {
    /// Milliseconds from start.
    pub ms: u64,
}

impl TimelinePosition {
    /// Create from milliseconds.
    pub fn from_ms(ms: u64) -> Self {
        Self { ms }
    }

    /// Create from seconds.
    pub fn from_secs(secs: f64) -> Self {
        Self { ms: (secs * 1000.0) as u64 }
    }

    /// Get as seconds.
    pub fn as_secs(&self) -> f64 {
        self.ms as f64 / 1000.0
    }
}

/// Video format.
#[derive(Debug, Clone, Copy, Default)]
pub enum VideoFormat {
    #[default]
    H264,
    H265,
    VP9,
    AV1,
    Raw,
}

/// Audio format.
#[derive(Debug, Clone, Copy, Default)]
pub enum AudioFormat {
    #[default]
    AAC,
    MP3,
    FLAC,
    WAV,
    Opus,
}

/// Video resolution.
#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const HD: Resolution = Resolution { width: 1280, height: 720 };
    pub const FHD: Resolution = Resolution { width: 1920, height: 1080 };
    pub const UHD: Resolution = Resolution { width: 3840, height: 2160 };
}

impl Default for Resolution {
    fn default() -> Self {
        Self::FHD
    }
}

/// Frame rate.
#[derive(Debug, Clone, Copy)]
pub struct FrameRate {
    pub numerator: u32,
    pub denominator: u32,
}

impl FrameRate {
    pub const FPS_24: FrameRate = FrameRate { numerator: 24, denominator: 1 };
    pub const FPS_30: FrameRate = FrameRate { numerator: 30, denominator: 1 };
    pub const FPS_60: FrameRate = FrameRate { numerator: 60, denominator: 1 };

    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        Self::FPS_30
    }
}

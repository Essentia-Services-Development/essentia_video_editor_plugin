//! Core video editor types.
//!
//! Fundamental data structures for video/audio format representation.

use essentia_time::Duration;
use essentia_utils::time;

/// Video resolution with 8K+ support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    /// Frame width in pixels.
    pub width:  u32,
    /// Frame height in pixels.
    pub height: u32,
}

impl Resolution {
    /// SD resolution (720x480).
    pub const SD: Resolution = Resolution { width: 720, height: 480 };
    /// HD resolution (1280x720).
    pub const HD: Resolution = Resolution { width: 1280, height: 720 };
    /// Full HD resolution (1920x1080).
    pub const FHD: Resolution = Resolution { width: 1920, height: 1080 };
    /// 2K resolution (2048x1080).
    pub const _2K: Resolution = Resolution { width: 2048, height: 1080 };
    /// QHD/1440p resolution (2560x1440).
    pub const QHD: Resolution = Resolution { width: 2560, height: 1440 };
    /// 4K UHD resolution (3840x2160).
    pub const UHD: Resolution = Resolution { width: 3840, height: 2160 };
    /// 4K DCI resolution (4096x2160).
    pub const _4K_DCI: Resolution = Resolution { width: 4096, height: 2160 };
    /// 8K UHD resolution (7680x4320).
    pub const _8K: Resolution = Resolution { width: 7680, height: 4320 };
    /// 12K resolution (11520x6480).
    pub const _12K: Resolution = Resolution { width: 11520, height: 6480 };

    /// Creates a new resolution.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Calculates aspect ratio as width/height.
    #[must_use]
    pub fn aspect_ratio(&self) -> f64 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f64 / self.height as f64
    }

    /// Returns total pixel count.
    #[must_use]
    pub const fn pixel_count(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Checks if resolution is 4K or higher.
    #[must_use]
    pub const fn is_ultra_hd(&self) -> bool {
        self.width >= 3840 && self.height >= 2160
    }

    /// Calculates scaled resolution maintaining aspect ratio.
    #[must_use]
    pub fn scaled_to_width(&self, target_width: u32) -> Self {
        let scale = target_width as f64 / self.width as f64;
        Self { width: target_width, height: (self.height as f64 * scale).round() as u32 }
    }

    /// Calculates scaled resolution maintaining aspect ratio.
    #[must_use]
    pub fn scaled_to_height(&self, target_height: u32) -> Self {
        let scale = target_height as f64 / self.height as f64;
        Self { width: (self.width as f64 * scale).round() as u32, height: target_height }
    }
}

impl Default for Resolution {
    fn default() -> Self {
        Self::FHD
    }
}

/// Frame rate representation using numerator/denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate {
    /// Frames per second numerator.
    pub numerator:   u32,
    /// Frames per second denominator.
    pub denominator: u32,
}

impl FrameRate {
    /// 23.976 fps (film with NTSC pulldown).
    pub const FPS_23_976: FrameRate = FrameRate { numerator: 24000, denominator: 1001 };
    /// 24 fps (cinema standard).
    pub const FPS_24: FrameRate = FrameRate { numerator: 24, denominator: 1 };
    /// 25 fps (PAL video).
    pub const FPS_25: FrameRate = FrameRate { numerator: 25, denominator: 1 };
    /// 29.97 fps (NTSC video).
    pub const FPS_29_97: FrameRate = FrameRate { numerator: 30000, denominator: 1001 };
    /// 30 fps (common web video).
    pub const FPS_30: FrameRate = FrameRate { numerator: 30, denominator: 1 };
    /// 48 fps (HFR cinema).
    pub const FPS_48: FrameRate = FrameRate { numerator: 48, denominator: 1 };
    /// 50 fps (PAL high frame rate).
    pub const FPS_50: FrameRate = FrameRate { numerator: 50, denominator: 1 };
    /// 59.94 fps (NTSC high frame rate).
    pub const FPS_59_94: FrameRate = FrameRate { numerator: 60000, denominator: 1001 };
    /// 60 fps (gaming/streaming standard).
    pub const FPS_60: FrameRate = FrameRate { numerator: 60, denominator: 1 };
    /// 120 fps (high performance displays).
    pub const FPS_120: FrameRate = FrameRate { numerator: 120, denominator: 1 };
    /// 144 fps (gaming displays).
    pub const FPS_144: FrameRate = FrameRate { numerator: 144, denominator: 1 };
    /// 240 fps (slow motion / high refresh).
    pub const FPS_240: FrameRate = FrameRate { numerator: 240, denominator: 1 };

    /// Creates a new frame rate.
    #[must_use]
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self { numerator, denominator }
    }

    /// Returns frame rate as floating point value.
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        if self.denominator == 0 {
            return 0.0;
        }
        self.numerator as f64 / self.denominator as f64
    }

    /// Returns frame duration in microseconds.
    #[must_use]
    pub fn frame_duration_us(&self) -> u64 {
        if self.numerator == 0 {
            return 0;
        }
        (self.denominator as u64 * 1_000_000) / self.numerator as u64
    }

    /// Returns frame duration in nanoseconds.
    #[must_use]
    pub fn frame_duration_ns(&self) -> u64 {
        if self.numerator == 0 {
            return 0;
        }
        (self.denominator as u64 * 1_000_000_000) / self.numerator as u64
    }

    /// Checks if this is a drop frame timecode rate.
    #[must_use]
    pub fn is_drop_frame(&self) -> bool {
        // Drop frame is used for 29.97 and 59.94 fps
        matches!(
            (self.numerator, self.denominator),
            (30000, 1001) | (60000, 1001)
        )
    }
}

impl Default for FrameRate {
    fn default() -> Self {
        Self::FPS_30
    }
}

/// Video codec formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VideoFormat {
    /// H.264/AVC codec.
    #[default]
    H264,
    /// H.265/HEVC codec (4K/8K optimized).
    H265,
    /// VP9 codec (WebM/YouTube).
    VP9,
    /// AV1 codec (next-gen open codec).
    AV1,
    /// ProRes codec (Apple professional).
    ProRes,
    /// DNxHD/DNxHR codec (Avid professional).
    DNxHD,
    /// Raw uncompressed video.
    Raw,
    /// MJPEG codec.
    MJPEG,
    /// Cineform codec.
    Cineform,
    /// FFV1 lossless codec.
    FFV1,
}

impl VideoFormat {
    /// Returns whether this codec supports 8K+ resolution.
    #[must_use]
    pub const fn supports_8k(&self) -> bool {
        matches!(self, Self::H265 | Self::AV1 | Self::Raw | Self::ProRes)
    }

    /// Returns whether this codec supports HDR.
    #[must_use]
    pub const fn supports_hdr(&self) -> bool {
        matches!(self, Self::H265 | Self::AV1 | Self::ProRes)
    }

    /// Returns whether this codec is lossless.
    #[must_use]
    pub const fn is_lossless(&self) -> bool {
        matches!(self, Self::Raw | Self::FFV1 | Self::ProRes)
    }
}

/// Audio codec formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioFormat {
    /// AAC codec (streaming standard).
    #[default]
    AAC,
    /// MP3 codec (legacy).
    MP3,
    /// FLAC lossless codec.
    FLAC,
    /// WAV uncompressed.
    WAV,
    /// Opus codec (WebRTC/streaming).
    Opus,
    /// AC3/Dolby Digital.
    AC3,
    /// EAC3/Dolby Digital Plus.
    EAC3,
    /// DTS audio.
    DTS,
    /// PCM uncompressed.
    PCM,
    /// Vorbis codec.
    Vorbis,
    /// ALAC (Apple Lossless).
    ALAC,
}

impl AudioFormat {
    /// Returns whether this format is lossless.
    #[must_use]
    pub const fn is_lossless(&self) -> bool {
        matches!(self, Self::FLAC | Self::WAV | Self::PCM | Self::ALAC)
    }

    /// Returns whether this format supports surround sound.
    #[must_use]
    pub const fn supports_surround(&self) -> bool {
        matches!(
            self,
            Self::AC3 | Self::EAC3 | Self::DTS | Self::FLAC | Self::PCM
        )
    }
}

/// Media type enumeration (video or audio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaKind {
    /// Video stream.
    Video,
    /// Audio stream.
    Audio,
    /// Subtitle/caption stream.
    Subtitle,
    /// Data stream (metadata, timecode, etc.).
    Data,
}

impl MediaKind {
    /// Checks if this is video media.
    #[must_use]
    pub const fn is_video(&self) -> bool {
        matches!(self, Self::Video)
    }

    /// Checks if this is audio media.
    #[must_use]
    pub const fn is_audio(&self) -> bool {
        matches!(self, Self::Audio)
    }

    /// Checks if this is subtitle media.
    #[must_use]
    pub const fn is_subtitle(&self) -> bool {
        matches!(self, Self::Subtitle)
    }
}

/// Timestamp information for frames and packets.
#[derive(Debug, Clone, Default)]
pub struct TimeInfo {
    /// Presentation timestamp (in timebase units).
    pub pts:          Option<i64>,
    /// Decode timestamp (in timebase units).
    pub dts:          Option<i64>,
    /// Duration (in timebase units).
    pub duration:     Option<u64>,
    /// Timebase numerator.
    pub timebase_num: u32,
    /// Timebase denominator.
    pub timebase_den: u32,
}

impl TimeInfo {
    /// Creates a new `TimeInfo` with the specified timebase.
    #[must_use]
    pub const fn new(timebase_num: u32, timebase_den: u32) -> Self {
        Self { pts: None, dts: None, duration: None, timebase_num, timebase_den }
    }

    /// Creates a `TimeInfo` with millisecond timebase.
    #[must_use]
    pub const fn milliseconds() -> Self {
        Self::new(1, 1000)
    }

    /// Creates a `TimeInfo` with microsecond timebase.
    #[must_use]
    pub const fn microseconds() -> Self {
        Self::new(1, 1_000_000)
    }

    /// Converts PTS to milliseconds.
    #[must_use]
    pub fn pts_ms(&self) -> Option<f64> {
        let pts = self.pts?;
        if self.timebase_den == 0 {
            return None;
        }
        Some((pts as f64 * self.timebase_num as f64 * 1000.0) / self.timebase_den as f64)
    }

    /// Converts duration to milliseconds.
    #[must_use]
    pub fn duration_ms(&self) -> Option<f64> {
        let duration = self.duration?;
        if self.timebase_den == 0 {
            return None;
        }
        Some((duration as f64 * self.timebase_num as f64 * 1000.0) / self.timebase_den as f64)
    }
}

/// Time position in timeline (millisecond precision).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TimePosition {
    /// Position in milliseconds.
    pub ms: u64,
}

impl TimePosition {
    /// Creates a new time position from milliseconds.
    #[must_use]
    pub const fn from_ms(ms: u64) -> Self {
        Self { ms }
    }

    /// Creates a time position from seconds.
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self { ms: secs * 1000 }
    }

    /// Creates a time position from frame number and frame rate.
    #[must_use]
    pub fn from_frame(frame: u64, frame_rate: &FrameRate) -> Self {
        if frame_rate.numerator == 0 {
            return Self { ms: 0 };
        }
        let ms = (frame * frame_rate.denominator as u64 * 1000) / frame_rate.numerator as u64;
        Self { ms }
    }

    /// Converts to frame number at the given frame rate.
    #[must_use]
    pub fn to_frame(&self, frame_rate: &FrameRate) -> u64 {
        if frame_rate.denominator == 0 {
            return 0;
        }
        (self.ms * frame_rate.numerator as u64) / (frame_rate.denominator as u64 * 1000)
    }

    /// Returns position in seconds.
    #[must_use]
    pub const fn as_secs(&self) -> u64 {
        self.ms / 1000
    }

    /// Returns position in seconds with fractional part.
    #[must_use]
    pub fn as_secs_f64(&self) -> f64 {
        self.ms as f64 / 1000.0
    }

    /// Parses a timecode string (HH:MM:SS:FF or HH:MM:SS.mmm).
    pub fn from_timecode(timecode: &str, frame_rate: &FrameRate) -> Option<Self> {
        let parts: Vec<&str> = timecode.split(':').collect();
        if parts.len() < 3 {
            return None;
        }

        let hours: u64 = parts[0].parse().ok()?;
        let minutes: u64 = parts[1].parse().ok()?;

        // Handle frames (HH:MM:SS:FF) or milliseconds (HH:MM:SS.mmm)
        let (seconds, frames_or_ms) = if parts.len() == 4 {
            let secs: u64 = parts[2].parse().ok()?;
            let frames: u64 = parts[3].parse().ok()?;
            (secs, Self::from_frame(frames, frame_rate).ms)
        } else if parts[2].contains('.') {
            let sec_parts: Vec<&str> = parts[2].split('.').collect();
            let secs: u64 = sec_parts[0].parse().ok()?;
            let ms: u64 = sec_parts.get(1)?.parse().ok()?;
            (secs, ms)
        } else {
            let secs: u64 = parts[2].parse().ok()?;
            (secs, 0)
        };

        let total_ms = hours * 3600000 + minutes * 60000 + seconds * 1000 + frames_or_ms;
        Some(Self { ms: total_ms })
    }

    /// Formats as timecode string (HH:MM:SS:FF).
    #[must_use]
    pub fn to_timecode(&self, frame_rate: &FrameRate) -> String {
        let total_secs = self.ms / 1000;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let remaining_ms = self.ms % 1000;

        let fps = frame_rate.as_f64();
        let frames = if fps > 0.0 {
            ((remaining_ms as f64 / 1000.0) * fps).round() as u64
        } else {
            0
        };

        format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames)
    }
}

impl std::ops::Add for TimePosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { ms: self.ms + rhs.ms }
    }
}

impl std::ops::Sub for TimePosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { ms: self.ms.saturating_sub(rhs.ms) }
    }
}

/// Aspect ratio representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AspectRatio {
    /// Width component.
    pub width:  u32,
    /// Height component.
    pub height: u32,
}

impl AspectRatio {
    /// Standard 4:3 aspect ratio.
    pub const STANDARD_4_3: AspectRatio = AspectRatio { width: 4, height: 3 };
    /// Widescreen 16:9 aspect ratio.
    pub const WIDESCREEN_16_9: AspectRatio = AspectRatio { width: 16, height: 9 };
    /// Cinema 21:9 aspect ratio.
    pub const CINEMA_21_9: AspectRatio = AspectRatio { width: 21, height: 9 };
    /// IMAX 1.43:1 aspect ratio.
    pub const IMAX: AspectRatio = AspectRatio { width: 143, height: 100 };
    /// Cinemascope 2.39:1 aspect ratio.
    pub const CINEMASCOPE: AspectRatio = AspectRatio { width: 239, height: 100 };
    /// Square 1:1 aspect ratio.
    pub const SQUARE: AspectRatio = AspectRatio { width: 1, height: 1 };

    /// Creates a new aspect ratio.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the ratio as a floating point value.
    #[must_use]
    pub fn as_f64(&self) -> f64 {
        if self.height == 0 {
            return 0.0;
        }
        self.width as f64 / self.height as f64
    }

    /// Creates an aspect ratio from a resolution.
    #[must_use]
    pub fn from_resolution(resolution: &Resolution) -> Self {
        let gcd = Self::gcd(resolution.width, resolution.height);
        if gcd == 0 {
            return Self::WIDESCREEN_16_9;
        }
        Self { width: resolution.width / gcd, height: resolution.height / gcd }
    }

    /// Calculates greatest common divisor.
    const fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { Self::gcd(b, a % b) }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::WIDESCREEN_16_9
    }
}

/// Real-world timestamp for project metadata and events.
///
/// Represents system time as Unix seconds for project operations
/// like autosave, modification tracking, and markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    /// Unix timestamp in seconds since epoch.
    secs: u64,
}

impl Timestamp {
    /// Creates a new timestamp from Unix seconds.
    #[must_use]
    pub const fn new(secs: u64) -> Self {
        Self { secs }
    }

    /// Creates a timestamp representing the current system time.
    #[must_use]
    pub fn now() -> Self {
        let secs = time::unix_seconds();
        Self { secs }
    }

    /// Creates a timestamp at Unix epoch (January 1, 1970).
    #[must_use]
    pub const fn epoch() -> Self {
        Self { secs: 0 }
    }

    /// Returns the Unix timestamp in seconds.
    #[must_use]
    pub const fn as_secs(&self) -> u64 {
        self.secs
    }

    /// Returns duration elapsed since this timestamp.
    ///
    /// Returns zero duration if the timestamp is in the future.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        if now.secs >= self.secs {
            Duration::from_secs(now.secs - self.secs)
        } else {
            Duration::ZERO
        }
    }

    /// Returns duration since another timestamp.
    ///
    /// Returns zero duration if `other` is after this timestamp.
    #[must_use]
    pub fn elapsed_since(&self, other: Self) -> Duration {
        if self.secs >= other.secs {
            Duration::from_secs(self.secs - other.secs)
        } else {
            Duration::ZERO
        }
    }

    /// Adds a duration to this timestamp.
    #[must_use]
    pub fn add_duration(&self, duration: Duration) -> Self {
        Self { secs: self.secs.saturating_add(duration.as_secs()) }
    }

    /// Subtracts a duration from this timestamp.
    #[must_use]
    pub fn sub_duration(&self, duration: Duration) -> Self {
        Self { secs: self.secs.saturating_sub(duration.as_secs()) }
    }

    /// Returns whether this timestamp is before another.
    #[must_use]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.secs < other.secs
    }

    /// Returns whether this timestamp is after another.
    #[must_use]
    pub const fn is_after(&self, other: &Self) -> bool {
        self.secs > other.secs
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

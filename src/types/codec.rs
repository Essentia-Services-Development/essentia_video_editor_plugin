//! Codec types and abstractions for video encoding/decoding.
//!
//! Inspired by rust-av's codec parameter patterns.

use std::collections::HashMap;

/// Codec identifier enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CodecId {
    // Video codecs
    /// H.264 / AVC.
    H264,
    /// H.265 / HEVC.
    H265,
    /// VP8.
    Vp8,
    /// VP9.
    Vp9,
    /// AV1.
    Av1,
    /// Apple ProRes.
    ProRes,
    /// DNxHD/DNxHR.
    DnxHd,
    /// JPEG 2000.
    Jpeg2000,
    /// FFV1 (lossless).
    Ffv1,
    /// Raw video (uncompressed).
    RawVideo,

    // Audio codecs
    /// AAC.
    Aac,
    /// MP3.
    Mp3,
    /// Opus.
    Opus,
    /// FLAC (lossless).
    Flac,
    /// PCM (uncompressed).
    Pcm,
    /// Vorbis.
    Vorbis,
    /// AC-3 / Dolby Digital.
    Ac3,
    /// E-AC-3 / Dolby Digital Plus.
    Eac3,
    /// DTS.
    Dts,

    // Other
    /// Unknown codec.
    #[default]
    Unknown,
}

impl CodecId {
    /// Returns true if this is a video codec.
    #[must_use]
    pub const fn is_video(&self) -> bool {
        matches!(
            self,
            Self::H264
                | Self::H265
                | Self::Vp8
                | Self::Vp9
                | Self::Av1
                | Self::ProRes
                | Self::DnxHd
                | Self::Jpeg2000
                | Self::Ffv1
                | Self::RawVideo
        )
    }

    /// Returns true if this is an audio codec.
    #[must_use]
    pub const fn is_audio(&self) -> bool {
        matches!(
            self,
            Self::Aac
                | Self::Mp3
                | Self::Opus
                | Self::Flac
                | Self::Pcm
                | Self::Vorbis
                | Self::Ac3
                | Self::Eac3
                | Self::Dts
        )
    }

    /// Returns true if this codec is lossless.
    #[must_use]
    pub const fn is_lossless(&self) -> bool {
        matches!(self, Self::Flac | Self::Ffv1 | Self::Pcm | Self::RawVideo)
    }

    /// Returns the codec name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::H264 => "H.264/AVC",
            Self::H265 => "H.265/HEVC",
            Self::Vp8 => "VP8",
            Self::Vp9 => "VP9",
            Self::Av1 => "AV1",
            Self::ProRes => "Apple ProRes",
            Self::DnxHd => "DNxHD/DNxHR",
            Self::Jpeg2000 => "JPEG 2000",
            Self::Ffv1 => "FFV1",
            Self::RawVideo => "Raw Video",
            Self::Aac => "AAC",
            Self::Mp3 => "MP3",
            Self::Opus => "Opus",
            Self::Flac => "FLAC",
            Self::Pcm => "PCM",
            Self::Vorbis => "Vorbis",
            Self::Ac3 => "AC-3",
            Self::Eac3 => "E-AC-3",
            Self::Dts => "DTS",
            Self::Unknown => "Unknown",
        }
    }
}

/// H.264 profile level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum H264Profile {
    /// Baseline profile.
    Baseline,
    /// Main profile.
    Main,
    /// High profile.
    High,
    /// High 10 profile (10-bit).
    High10,
    /// High 4:2:2 profile.
    High422,
    /// High 4:4:4 profile.
    High444,
}

/// H.265 profile level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum H265Profile {
    /// Main profile.
    Main,
    /// Main 10 profile (10-bit).
    Main10,
    /// Main 12 profile (12-bit).
    Main12,
    /// Main Still Picture.
    MainStillPicture,
    /// Main 4:2:2 10-bit.
    Main42210,
    /// Main 4:4:4.
    Main444,
}

/// AV1 profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Av1Profile {
    /// Main profile (8-bit, 4:2:0).
    Main,
    /// High profile (8-bit, 4:4:4).
    High,
    /// Professional profile (up to 12-bit).
    Professional,
}

/// ProRes codec variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProResProfile {
    /// Proxy quality.
    Proxy,
    /// LT (Light) quality.
    Lt,
    /// Standard quality.
    Standard,
    /// HQ (High Quality).
    Hq,
    /// 4444 (with alpha).
    P4444,
    /// 4444 XQ (highest quality).
    P4444xq,
}

/// Rate control mode for encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateControlMode {
    /// Constant Quality Factor.
    Cqf(u8),
    /// Constant Rate Factor.
    Crf(u8),
    /// Constant Bitrate.
    Cbr,
    /// Variable Bitrate.
    Vbr,
    /// Average Bitrate.
    Abr,
    /// Two-pass encoding.
    TwoPass,
}

/// Video encoder configuration.
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    /// Codec to use.
    pub codec:             CodecId,
    /// Frame width.
    pub width:             u32,
    /// Frame height.
    pub height:            u32,
    /// Target bitrate in bits/second (for VBR/ABR).
    pub bitrate:           u64,
    /// Max bitrate in bits/second.
    pub max_bitrate:       Option<u64>,
    /// Buffer size in bits.
    pub buffer_size:       Option<u64>,
    /// Rate control mode.
    pub rate_control:      RateControlMode,
    /// Keyframe interval (GOP size).
    pub keyframe_interval: u32,
    /// B-frames count.
    pub b_frames:          u8,
    /// Reference frames.
    pub ref_frames:        u8,
    /// Preset (e.g., "ultrafast", "medium", "veryslow").
    pub preset:            String,
    /// Tuning option (e.g., "film", "animation").
    pub tune:              Option<String>,
    /// Additional codec-specific options.
    pub extra_options:     HashMap<String, String>,
}

impl EncoderConfig {
    /// Creates a new encoder config with defaults.
    #[must_use]
    pub fn new(codec: CodecId, width: u32, height: u32) -> Self {
        Self {
            codec,
            width,
            height,
            bitrate: 10_000_000, // 10 Mbps default
            max_bitrate: None,
            buffer_size: None,
            rate_control: RateControlMode::Crf(23),
            keyframe_interval: 250,
            b_frames: 2,
            ref_frames: 3,
            preset: String::from("medium"),
            tune: None,
            extra_options: HashMap::new(),
        }
    }

    /// Creates a high-quality encoder config.
    #[must_use]
    pub fn high_quality(codec: CodecId, width: u32, height: u32) -> Self {
        Self {
            codec,
            width,
            height,
            bitrate: 50_000_000, // 50 Mbps
            max_bitrate: Some(80_000_000),
            buffer_size: Some(100_000_000),
            rate_control: RateControlMode::Crf(18),
            keyframe_interval: 120,
            b_frames: 4,
            ref_frames: 5,
            preset: String::from("slow"),
            tune: Some(String::from("film")),
            extra_options: HashMap::new(),
        }
    }

    /// Creates a fast encoding config for preview.
    #[must_use]
    pub fn fast_preview(codec: CodecId, width: u32, height: u32) -> Self {
        Self {
            codec,
            width,
            height,
            bitrate: 2_000_000, // 2 Mbps
            max_bitrate: None,
            buffer_size: None,
            rate_control: RateControlMode::Crf(28),
            keyframe_interval: 30,
            b_frames: 0,
            ref_frames: 1,
            preset: String::from("ultrafast"),
            tune: Some(String::from("zerolatency")),
            extra_options: HashMap::new(),
        }
    }

    /// Sets an extra codec option.
    pub fn set_option(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra_options.insert(key.into(), value.into());
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self::new(CodecId::H264, 1920, 1080)
    }
}

/// Video decoder configuration.
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Codec to use.
    pub codec:               CodecId,
    /// Hardware acceleration preference.
    pub hw_accel:            HwAccelPreference,
    /// Thread count (0 = auto).
    pub threads:             u32,
    /// Low-latency mode.
    pub low_latency:         bool,
    /// Skip loop filter (deblocking).
    pub skip_loop_filter:    bool,
    /// Skip IDCT/rescaling.
    pub skip_idct:           bool,
    /// Frame drop threshold (-1 = none).
    pub framedrop_threshold: i32,
    /// Additional codec-specific options.
    pub extra_options:       HashMap<String, String>,
}

impl DecoderConfig {
    /// Creates a new decoder config with defaults.
    #[must_use]
    pub fn new(codec: CodecId) -> Self {
        Self {
            codec,
            hw_accel: HwAccelPreference::Auto,
            threads: 0,
            low_latency: false,
            skip_loop_filter: false,
            skip_idct: false,
            framedrop_threshold: -1,
            extra_options: HashMap::new(),
        }
    }

    /// Creates a low-latency decoder config.
    #[must_use]
    pub fn low_latency(codec: CodecId) -> Self {
        Self {
            codec,
            hw_accel: HwAccelPreference::Auto,
            threads: 1,
            low_latency: true,
            skip_loop_filter: false,
            skip_idct: false,
            framedrop_threshold: 0,
            extra_options: HashMap::new(),
        }
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self::new(CodecId::Unknown)
    }
}

/// Hardware acceleration preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HwAccelPreference {
    /// Automatic selection.
    #[default]
    Auto,
    /// Prefer NVIDIA NVENC/NVDEC.
    Nvenc,
    /// Prefer AMD VCE/VCN.
    Amd,
    /// Prefer Intel QuickSync.
    QuickSync,
    /// Prefer Vulkan Video.
    Vulkan,
    /// Software only (no hardware acceleration).
    None,
}

impl HwAccelPreference {
    /// Returns whether this preference allows hardware acceleration.
    #[must_use]
    pub const fn allows_hw(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Codec parameters container.
#[derive(Debug, Clone)]
pub struct CodecParams {
    /// Codec ID.
    pub codec_id:    CodecId,
    /// Extradata (codec-specific header).
    pub extradata:   Option<Vec<u8>>,
    /// Width for video.
    pub width:       Option<u32>,
    /// Height for video.
    pub height:      Option<u32>,
    /// Bitrate.
    pub bitrate:     Option<u64>,
    /// Sample rate for audio.
    pub sample_rate: Option<u32>,
    /// Channel count for audio.
    pub channels:    Option<u8>,
    /// Bits per sample.
    pub bits:        Option<u8>,
}

impl CodecParams {
    /// Creates new empty codec params.
    #[must_use]
    pub const fn new(codec_id: CodecId) -> Self {
        Self {
            codec_id,
            extradata: None,
            width: None,
            height: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
            bits: None,
        }
    }

    /// Creates video codec params.
    #[must_use]
    pub fn video(codec_id: CodecId, width: u32, height: u32) -> Self {
        Self {
            codec_id,
            extradata: None,
            width: Some(width),
            height: Some(height),
            bitrate: None,
            sample_rate: None,
            channels: None,
            bits: None,
        }
    }

    /// Creates audio codec params.
    #[must_use]
    pub fn audio(codec_id: CodecId, sample_rate: u32, channels: u8) -> Self {
        Self {
            codec_id,
            extradata: None,
            width: None,
            height: None,
            bitrate: None,
            sample_rate: Some(sample_rate),
            channels: Some(channels),
            bits: None,
        }
    }
}

impl Default for CodecParams {
    fn default() -> Self {
        Self::new(CodecId::Unknown)
    }
}

//! Video/audio format types, codecs, and encoding settings.

use crate::types::{FrameRate, Resolution};

/// Unique identifier for an export job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExportJobId(u64);

impl ExportJobId {
    /// Creates a new export job ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    #[must_use]
    pub const fn inner(&self) -> u64 {
        self.0
    }
}

/// Video container format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContainerFormat {
    /// MP4 (MPEG-4 Part 14).
    #[default]
    Mp4,
    /// MOV (QuickTime).
    Mov,
    /// MKV (Matroska).
    Mkv,
    /// WebM.
    WebM,
    /// AVI.
    Avi,
    /// MPEG Transport Stream.
    MpegTs,
    /// Raw video (no container).
    Raw,
}

impl ContainerFormat {
    /// Returns the file extension for this format.
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mov => "mov",
            Self::Mkv => "mkv",
            Self::WebM => "webm",
            Self::Avi => "avi",
            Self::MpegTs => "ts",
            Self::Raw => "raw",
        }
    }

    /// Returns the MIME type.
    #[must_use]
    pub const fn mime_type(&self) -> &'static str {
        match self {
            Self::Mp4 => "video/mp4",
            Self::Mov => "video/quicktime",
            Self::Mkv => "video/x-matroska",
            Self::WebM => "video/webm",
            Self::Avi => "video/x-msvideo",
            Self::MpegTs => "video/mp2t",
            Self::Raw => "video/raw",
        }
    }
}

/// Video codec for encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VideoCodec {
    /// H.264/AVC.
    #[default]
    H264,
    /// H.265/HEVC.
    H265,
    /// VP8.
    Vp8,
    /// VP9.
    Vp9,
    /// AV1.
    Av1,
    /// ProRes.
    ProRes(ProResProfile),
    /// DNxHD/DNxHR.
    DnxHd(DnxProfile),
    /// Uncompressed.
    Uncompressed,
}

/// ProRes profile variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProResProfile {
    /// ProRes 422 Proxy.
    Proxy,
    /// ProRes 422 LT.
    Lt,
    /// ProRes 422.
    #[default]
    Standard,
    /// ProRes 422 HQ.
    Hq,
    /// ProRes 4444.
    FourFour,
    /// ProRes 4444 XQ.
    FourFourXq,
}

/// DNxHD/DNxHR profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnxProfile {
    /// DNxHD 36 (proxy).
    Dnx36,
    /// DNxHD 145/220 (standard).
    #[default]
    Dnx145,
    /// DNxHR SQ.
    DnxHrSq,
    /// DNxHR HQ.
    DnxHrHq,
    /// DNxHR HQX.
    DnxHrHqx,
    /// DNxHR 444.
    DnxHr444,
}

/// Audio codec for encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioCodec {
    /// AAC.
    #[default]
    Aac,
    /// MP3.
    Mp3,
    /// Opus.
    Opus,
    /// Vorbis.
    Vorbis,
    /// FLAC.
    Flac,
    /// PCM (uncompressed).
    Pcm,
    /// AC-3.
    Ac3,
    /// E-AC-3.
    Eac3,
}

/// Encoding rate control mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RateControl {
    /// Constant bitrate.
    Cbr,
    /// Variable bitrate.
    #[default]
    Vbr,
    /// Constant quality (CRF/CQP).
    ConstantQuality,
    /// Two-pass encoding.
    TwoPass,
}

/// Hardware acceleration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HardwareAccel {
    /// Software encoding only.
    #[default]
    None,
    /// NVIDIA NVENC.
    Nvenc,
    /// Intel Quick Sync.
    QuickSync,
    /// AMD VCE/VCN.
    AmdVce,
    /// Apple VideoToolbox.
    VideoToolbox,
    /// Vulkan Video.
    VulkanVideo,
}

/// Encoding preset (speed vs quality trade-off).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EncodingPreset {
    /// Fastest encoding, lowest quality.
    Ultrafast,
    /// Very fast encoding.
    Superfast,
    /// Fast encoding.
    Veryfast,
    /// Faster than default.
    Faster,
    /// Fast encoding.
    Fast,
    /// Balanced.
    #[default]
    Medium,
    /// Slower, better quality.
    Slow,
    /// Even slower, even better quality.
    Slower,
    /// Very slow, high quality.
    Veryslow,
    /// Slowest, best quality.
    Placebo,
}

/// Pixel format for video encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PixelFormat {
    /// YUV 4:2:0 8-bit.
    #[default]
    Yuv420p,
    /// YUV 4:2:2 8-bit.
    Yuv422p,
    /// YUV 4:4:4 8-bit.
    Yuv444p,
    /// YUV 4:2:0 10-bit.
    Yuv420p10,
    /// YUV 4:2:2 10-bit.
    Yuv422p10,
    /// YUV 4:4:4 10-bit.
    Yuv444p10,
    /// RGB 24-bit.
    Rgb24,
    /// RGBA 32-bit.
    Rgba,
}

/// Video encoding settings.
#[derive(Debug, Clone)]
pub struct VideoEncodingSettings {
    /// Video codec.
    pub codec:        VideoCodec,
    /// Resolution.
    pub resolution:   Resolution,
    /// Frame rate.
    pub frame_rate:   FrameRate,
    /// Bitrate in kbps (for CBR/VBR).
    pub bitrate:      u32,
    /// Quality value (0-51 for CRF, codec-specific).
    pub quality:      u8,
    /// Rate control mode.
    pub rate_control: RateControl,
    /// Hardware acceleration.
    pub hw_accel:     HardwareAccel,
    /// B-frame count.
    pub b_frames:     u8,
    /// GOP size (keyframe interval).
    pub gop_size:     u32,
    /// Encoding preset (speed vs quality).
    pub preset:       EncodingPreset,
    /// Pixel format.
    pub pixel_format: PixelFormat,
}

impl Default for VideoEncodingSettings {
    fn default() -> Self {
        Self {
            codec:        VideoCodec::default(),
            resolution:   Resolution::new(1920, 1080),
            frame_rate:   FrameRate::new(30, 1),
            bitrate:      10000,
            quality:      23,
            rate_control: RateControl::default(),
            hw_accel:     HardwareAccel::default(),
            b_frames:     2,
            gop_size:     250,
            preset:       EncodingPreset::default(),
            pixel_format: PixelFormat::default(),
        }
    }
}

/// Audio encoding settings.
#[derive(Debug, Clone)]
pub struct AudioEncodingSettings {
    /// Audio codec.
    pub codec:       AudioCodec,
    /// Bitrate in kbps.
    pub bitrate:     u32,
    /// Sample rate.
    pub sample_rate: u32,
    /// Channel count.
    pub channels:    u8,
}

impl Default for AudioEncodingSettings {
    fn default() -> Self {
        Self {
            codec:       AudioCodec::default(),
            bitrate:     192,
            sample_rate: 48000,
            channels:    2,
        }
    }
}

/// Export settings combining all encoding options.
#[derive(Debug, Clone, Default)]
pub struct ExportSettings {
    /// Container format.
    pub container:   ContainerFormat,
    /// Video settings.
    pub video:       VideoEncodingSettings,
    /// Audio settings.
    pub audio:       AudioEncodingSettings,
    /// Output file path.
    pub output_path: String,
    /// Range to export (None = entire timeline).
    pub range:       Option<(crate::types::TimePosition, crate::types::TimePosition)>,
    /// Enable multi-pass encoding.
    pub multi_pass:  bool,
    /// Metadata to embed.
    pub metadata:    ExportMetadata,
}

/// Metadata to embed in exported file.
#[derive(Debug, Clone, Default)]
pub struct ExportMetadata {
    /// Title.
    pub title:     Option<String>,
    /// Artist/author.
    pub artist:    Option<String>,
    /// Album/collection.
    pub album:     Option<String>,
    /// Year.
    pub year:      Option<u16>,
    /// Comment.
    pub comment:   Option<String>,
    /// Copyright notice.
    pub copyright: Option<String>,
    /// Custom key-value metadata.
    pub custom:    Vec<(String, String)>,
}

/// Export job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ExportStatus {
    /// Job is queued, waiting to start.
    #[default]
    Queued,
    /// Job is preparing (analyzing, building graph).
    Preparing,
    /// First pass in progress (for multi-pass).
    FirstPass,
    /// Second pass in progress.
    SecondPass,
    /// Encoding in progress.
    Encoding,
    /// Finalizing (muxing, writing metadata).
    Finalizing,
    /// Export completed successfully.
    Completed,
    /// Export failed.
    Failed,
    /// Export was cancelled.
    Cancelled,
    /// Export is paused.
    Paused,
}

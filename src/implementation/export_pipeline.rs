//! Export Pipeline for Essentia Video Editor Plugin
//! GAP-220-B-003: Video Export System
//!
//! Features: Render queue, format encoding, codec configuration,
//! progress tracking, and multi-format export.

use crate::{
    errors::{VideoEditorError, VideoEditorResult},
    types::{FrameRate, Resolution, TimePosition, Timestamp},
};

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
    pub range:       Option<(TimePosition, TimePosition)>,
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

/// Progress information for an export job.
#[derive(Debug, Clone)]
pub struct ExportProgress {
    /// Current status.
    pub status:          ExportStatus,
    /// Progress percentage (0.0 to 1.0).
    pub progress:        f64,
    /// Frames encoded so far.
    pub frames_encoded:  u64,
    /// Total frames to encode.
    pub total_frames:    u64,
    /// Current frame rate (fps).
    pub encoding_fps:    f64,
    /// Estimated time remaining in seconds.
    pub eta_seconds:     Option<f64>,
    /// Current file size in bytes.
    pub current_size:    u64,
    /// Estimated final size in bytes.
    pub estimated_size:  Option<u64>,
    /// Current bitrate in kbps.
    pub current_bitrate: f64,
    /// Error message if failed.
    pub error_message:   Option<String>,
}

impl ExportProgress {
    /// Creates a new progress tracker.
    #[must_use]
    pub fn new(total_frames: u64) -> Self {
        Self {
            status: ExportStatus::Queued,
            progress: 0.0,
            frames_encoded: 0,
            total_frames,
            encoding_fps: 0.0,
            eta_seconds: None,
            current_size: 0,
            estimated_size: None,
            current_bitrate: 0.0,
            error_message: None,
        }
    }

    /// Updates progress with new frame count.
    pub fn update(&mut self, frames_encoded: u64, elapsed_seconds: f64) {
        self.frames_encoded = frames_encoded;

        if self.total_frames > 0 {
            self.progress = frames_encoded as f64 / self.total_frames as f64;
        }

        if elapsed_seconds > 0.0 {
            self.encoding_fps = frames_encoded as f64 / elapsed_seconds;

            if self.encoding_fps > 0.0 {
                let remaining_frames = self.total_frames.saturating_sub(frames_encoded);
                self.eta_seconds = Some(remaining_frames as f64 / self.encoding_fps);
            }
        }
    }

    /// Returns whether the export is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            ExportStatus::Completed | ExportStatus::Failed | ExportStatus::Cancelled
        )
    }
}

/// An export job in the render queue.
#[derive(Debug)]
pub struct ExportJob {
    /// Job identifier.
    id:         ExportJobId,
    /// Export settings.
    settings:   ExportSettings,
    /// Progress information.
    progress:   ExportProgress,
    /// Project ID this export is from.
    project_id: u64,
    /// When the job was created.
    created_at: Timestamp,
    /// When encoding started.
    started_at: Option<Timestamp>,
    /// When encoding completed.
    ended_at:   Option<Timestamp>,
    /// Priority (higher = more important).
    priority:   i32,
}

impl ExportJob {
    /// Creates a new export job.
    #[must_use]
    pub fn new(
        id: ExportJobId, project_id: u64, settings: ExportSettings, total_frames: u64,
    ) -> Self {
        Self {
            id,
            settings,
            progress: ExportProgress::new(total_frames),
            project_id,
            created_at: Timestamp::now(),
            started_at: None,
            ended_at: None,
            priority: 0,
        }
    }

    /// Returns the job ID.
    #[must_use]
    pub const fn id(&self) -> ExportJobId {
        self.id
    }

    /// Returns the export settings.
    #[must_use]
    pub fn settings(&self) -> &ExportSettings {
        &self.settings
    }

    /// Returns the current progress.
    #[must_use]
    pub fn progress(&self) -> &ExportProgress {
        &self.progress
    }

    /// Returns mutable progress.
    pub fn progress_mut(&mut self) -> &mut ExportProgress {
        &mut self.progress
    }

    /// Returns the project ID.
    #[must_use]
    pub const fn project_id(&self) -> u64 {
        self.project_id
    }

    /// Returns when the job was created.
    #[must_use]
    pub const fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns the job priority.
    #[must_use]
    pub const fn priority(&self) -> i32 {
        self.priority
    }

    /// Sets the job priority.
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    /// Marks the job as started.
    pub fn start(&mut self) {
        self.started_at = Some(Timestamp::now());
        self.progress.status = ExportStatus::Preparing;
    }

    /// Marks the job as completed.
    pub fn complete(&mut self) {
        self.ended_at = Some(Timestamp::now());
        self.progress.status = ExportStatus::Completed;
        self.progress.progress = 1.0;
    }

    /// Marks the job as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.ended_at = Some(Timestamp::now());
        self.progress.status = ExportStatus::Failed;
        self.progress.error_message = Some(error.into());
    }

    /// Cancels the job.
    pub fn cancel(&mut self) {
        self.ended_at = Some(Timestamp::now());
        self.progress.status = ExportStatus::Cancelled;
    }

    /// Returns elapsed encoding time.
    #[must_use]
    pub fn elapsed_time(&self) -> Option<f64> {
        let start = self.started_at?;
        let end = self.ended_at.unwrap_or_else(Timestamp::now);
        Some(end.elapsed_since(start).as_secs_f64())
    }
}

/// Export queue manager.
pub struct ExportQueue {
    /// All export jobs.
    jobs:           Vec<ExportJob>,
    /// Next job ID.
    next_id:        u64,
    /// Currently encoding job.
    current:        Option<ExportJobId>,
    /// Maximum concurrent exports.
    max_concurrent: usize,
    /// Active job count.
    active_count:   usize,
}

impl ExportQueue {
    /// Creates a new export queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            jobs:           Vec::new(),
            next_id:        1,
            current:        None,
            max_concurrent: 1,
            active_count:   0,
        }
    }

    /// Generates a new job ID.
    fn next_id(&mut self) -> ExportJobId {
        let id = ExportJobId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds a new export job to the queue.
    pub fn add_job(
        &mut self, project_id: u64, settings: ExportSettings, total_frames: u64,
    ) -> ExportJobId {
        let id = self.next_id();
        let job = ExportJob::new(id, project_id, settings, total_frames);
        self.jobs.push(job);
        id
    }

    /// Removes a job from the queue.
    pub fn remove_job(&mut self, id: ExportJobId) -> bool {
        if let Some(pos) = self.jobs.iter().position(|j| j.id() == id) {
            let job = &self.jobs[pos];
            // Can't remove active jobs
            if !job.progress().is_complete()
                && !matches!(job.progress().status, ExportStatus::Queued)
            {
                return false;
            }
            self.jobs.remove(pos);
            true
        } else {
            false
        }
    }

    /// Gets a job by ID.
    #[must_use]
    pub fn get_job(&self, id: ExportJobId) -> Option<&ExportJob> {
        self.jobs.iter().find(|j| j.id() == id)
    }

    /// Gets a mutable job by ID.
    pub fn get_job_mut(&mut self, id: ExportJobId) -> Option<&mut ExportJob> {
        self.jobs.iter_mut().find(|j| j.id() == id)
    }

    /// Returns all jobs.
    #[must_use]
    pub fn jobs(&self) -> &[ExportJob] {
        &self.jobs
    }

    /// Returns queued jobs in priority order.
    #[must_use]
    pub fn queued_jobs(&self) -> Vec<&ExportJob> {
        let mut queued: Vec<_> = self
            .jobs
            .iter()
            .filter(|j| matches!(j.progress().status, ExportStatus::Queued))
            .collect();
        queued.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        queued
    }

    /// Returns active jobs.
    #[must_use]
    pub fn active_jobs(&self) -> Vec<&ExportJob> {
        self.jobs
            .iter()
            .filter(|j| {
                matches!(
                    j.progress().status,
                    ExportStatus::Preparing
                        | ExportStatus::FirstPass
                        | ExportStatus::SecondPass
                        | ExportStatus::Encoding
                        | ExportStatus::Finalizing
                )
            })
            .collect()
    }

    /// Returns completed jobs.
    #[must_use]
    pub fn completed_jobs(&self) -> Vec<&ExportJob> {
        self.jobs
            .iter()
            .filter(|j| matches!(j.progress().status, ExportStatus::Completed))
            .collect()
    }

    /// Returns failed jobs.
    #[must_use]
    pub fn failed_jobs(&self) -> Vec<&ExportJob> {
        self.jobs
            .iter()
            .filter(|j| matches!(j.progress().status, ExportStatus::Failed))
            .collect()
    }

    /// Starts the next queued job if possible.
    pub fn start_next(&mut self) -> Option<ExportJobId> {
        if self.active_count >= self.max_concurrent {
            return None;
        }

        let queued = self.queued_jobs();
        let next_id = queued.first().map(|j| j.id())?;

        if let Some(job) = self.get_job_mut(next_id) {
            job.start();
            self.current = Some(next_id);
            self.active_count += 1;
            Some(next_id)
        } else {
            None
        }
    }

    /// Sets maximum concurrent exports.
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    /// Cancels a job.
    pub fn cancel_job(&mut self, id: ExportJobId) -> VideoEditorResult<()> {
        let job = self
            .get_job_mut(id)
            .ok_or_else(|| VideoEditorError::Export("Job not found".into()))?;

        if job.progress().is_complete() {
            return Err(VideoEditorError::Export(
                "Cannot cancel completed job".into(),
            ));
        }

        job.cancel();
        if self.current == Some(id) {
            self.current = None;
            self.active_count = self.active_count.saturating_sub(1);
        }

        Ok(())
    }

    /// Retries a failed job.
    pub fn retry_job(&mut self, id: ExportJobId) -> VideoEditorResult<ExportJobId> {
        let job = self
            .get_job(id)
            .ok_or_else(|| VideoEditorError::Export("Job not found".into()))?;

        if !matches!(
            job.progress().status,
            ExportStatus::Failed | ExportStatus::Cancelled
        ) {
            return Err(VideoEditorError::Export(
                "Can only retry failed/cancelled jobs".into(),
            ));
        }

        // Clone settings and create new job
        let settings = job.settings().clone();
        let total_frames = job.progress().total_frames;
        let project_id = job.project_id();

        Ok(self.add_job(project_id, settings, total_frames))
    }

    /// Clears completed jobs from the queue.
    pub fn clear_completed(&mut self) {
        self.jobs.retain(|j| !matches!(j.progress().status, ExportStatus::Completed));
    }

    /// Clears failed jobs from the queue.
    pub fn clear_failed(&mut self) {
        self.jobs.retain(|j| {
            !matches!(
                j.progress().status,
                ExportStatus::Failed | ExportStatus::Cancelled
            )
        });
    }
}

impl Default for ExportQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Export preset for common configurations.
#[derive(Debug, Clone)]
pub struct ExportPreset {
    /// Preset name.
    pub name:        String,
    /// Description.
    pub description: String,
    /// Category.
    pub category:    PresetCategory,
    /// Export settings.
    pub settings:    ExportSettings,
}

/// Categories for export presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PresetCategory {
    /// Social media platforms.
    #[default]
    Social,
    /// Broadcast/TV.
    Broadcast,
    /// Cinema/Film.
    Cinema,
    /// Web/streaming.
    Web,
    /// Archive/master.
    Archive,
    /// Mobile devices.
    Mobile,
    /// Custom presets.
    Custom,
}

impl ExportPreset {
    /// Creates a streaming platform HD (1080p) preset.
    ///
    /// Optimized settings for video sharing platforms.
    #[must_use]
    pub fn streaming_hd() -> Self {
        Self {
            name:        "Streaming HD 1080p".into(),
            description: "Optimized for video sharing platforms".into(),
            category:    PresetCategory::Social,
            settings:    ExportSettings {
                container: ContainerFormat::Mp4,
                video: VideoEncodingSettings {
                    codec: VideoCodec::H264,
                    resolution: Resolution::new(1920, 1080),
                    frame_rate: FrameRate::new(30, 1),
                    bitrate: 12000,
                    quality: 20,
                    rate_control: RateControl::Vbr,
                    preset: EncodingPreset::Slow,
                    ..Default::default()
                },
                audio: AudioEncodingSettings {
                    codec:       AudioCodec::Aac,
                    bitrate:     256,
                    sample_rate: 48000,
                    channels:    2,
                },
                ..Default::default()
            },
        }
    }

    /// Alias for streaming_hd for backward compatibility.
    #[must_use]
    #[deprecated(since = "1.0.0", note = "Use streaming_hd() instead")]
    pub fn youtube_1080p() -> Self {
        Self::streaming_hd()
    }

    /// Creates a streaming platform 4K preset.
    ///
    /// Optimized settings for 4K video sharing platforms.
    #[must_use]
    pub fn streaming_4k() -> Self {
        Self {
            name:        "Streaming 4K".into(),
            description: "Optimized for 4K video sharing platforms".into(),
            category:    PresetCategory::Social,
            settings:    ExportSettings {
                container: ContainerFormat::Mp4,
                video: VideoEncodingSettings {
                    codec: VideoCodec::H264,
                    resolution: Resolution::new(3840, 2160),
                    frame_rate: FrameRate::new(30, 1),
                    bitrate: 45000,
                    quality: 18,
                    rate_control: RateControl::Vbr,
                    preset: EncodingPreset::Slow,
                    ..Default::default()
                },
                audio: AudioEncodingSettings {
                    codec:       AudioCodec::Aac,
                    bitrate:     384,
                    sample_rate: 48000,
                    channels:    2,
                },
                ..Default::default()
            },
        }
    }

    /// Alias for streaming_4k for backward compatibility.
    #[must_use]
    #[deprecated(since = "1.0.0", note = "Use streaming_4k() instead")]
    pub fn youtube_4k() -> Self {
        Self::streaming_4k()
    }

    /// Creates a ProRes 422 HQ archive preset.
    #[must_use]
    pub fn prores_hq() -> Self {
        Self {
            name:        "ProRes 422 HQ".into(),
            description: "High quality archive format".into(),
            category:    PresetCategory::Archive,
            settings:    ExportSettings {
                container: ContainerFormat::Mov,
                video: VideoEncodingSettings {
                    codec: VideoCodec::ProRes(ProResProfile::Hq),
                    resolution: Resolution::new(1920, 1080),
                    frame_rate: FrameRate::new(24, 1),
                    bitrate: 220000, // ~220 Mbps
                    quality: 0,
                    rate_control: RateControl::ConstantQuality,
                    pixel_format: PixelFormat::Yuv422p10,
                    ..Default::default()
                },
                audio: AudioEncodingSettings {
                    codec:       AudioCodec::Pcm,
                    bitrate:     0, // N/A for PCM
                    sample_rate: 48000,
                    channels:    2,
                },
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_queue() {
        let mut queue = ExportQueue::new();

        let settings = ExportSettings::default();
        let id = queue.add_job(1, settings, 1000);

        assert!(queue.get_job(id).is_some());
        assert_eq!(queue.queued_jobs().len(), 1);
    }

    #[test]
    fn test_export_progress() {
        let mut progress = ExportProgress::new(1000);
        progress.update(500, 10.0);

        assert!((progress.progress - 0.5).abs() < 0.001);
        assert!(progress.encoding_fps > 0.0);
        assert!(progress.eta_seconds.is_some());
    }

    #[test]
    fn test_container_format() {
        let mp4 = ContainerFormat::Mp4;
        assert_eq!(mp4.extension(), "mp4");
        assert_eq!(mp4.mime_type(), "video/mp4");
    }

    #[test]
    fn test_export_preset() {
        let preset = ExportPreset::streaming_hd();
        assert_eq!(preset.settings.video.resolution.width, 1920);
        assert_eq!(preset.settings.video.resolution.height, 1080);
    }
}

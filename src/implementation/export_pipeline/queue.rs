//! Export queue manager and presets.

use crate::errors::{VideoEditorError, VideoEditorResult};
use crate::types::Resolution;

use super::formats::{
    AudioCodec, AudioEncodingSettings, ContainerFormat, EncodingPreset, ExportJobId,
    ExportSettings, ExportStatus, PixelFormat, ProResProfile, RateControl, VideoCodec,
    VideoEncodingSettings,
};
use super::job::ExportJob;
use crate::types::FrameRate;

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

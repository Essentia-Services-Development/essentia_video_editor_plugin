//! Export job and progress tracking.

use super::formats::{ExportJobId, ExportSettings, ExportStatus};
use crate::types::Timestamp;

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
    pub(super) id:         ExportJobId,
    /// Export settings.
    pub(super) settings:   ExportSettings,
    /// Progress information.
    pub(super) progress:   ExportProgress,
    /// Project ID this export is from.
    pub(super) project_id: u64,
    /// When the job was created.
    pub(super) created_at: Timestamp,
    /// When encoding started.
    pub(super) started_at: Option<Timestamp>,
    /// When encoding completed.
    pub(super) ended_at:   Option<Timestamp>,
    /// Priority (higher = more important).
    pub(super) priority:   i32,
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

//! Export Pipeline for Essentia Video Editor Plugin
//! GAP-220-B-003: Video Export System
//!
//! Features: Render queue, format encoding, codec configuration,
//! progress tracking, and multi-format export.

mod formats;
mod job;
mod queue;

#[cfg(test)]
mod tests {
    use super::{
        formats::*,
        job::{ExportJob, ExportProgress},
        queue::{ExportPreset, ExportQueue},
    };

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

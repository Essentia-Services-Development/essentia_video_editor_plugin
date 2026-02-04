//! Video editor plugin implementation.

use super::{AssetLibrary, EffectsPipeline, GpuPipeline, TimelineManager, VideoEditorConfig};
use crate::types::TrackType;

/// Main video editor plugin interface.
pub struct VideoEditorPlugin {
    config:   VideoEditorConfig,
    timeline: TimelineManager,
    assets:   AssetLibrary,
    effects:  EffectsPipeline,
    gpu:      GpuPipeline,
}

impl VideoEditorPlugin {
    /// Create a new video editor plugin.
    pub fn new(config: VideoEditorConfig) -> Self {
        let gpu = GpuPipeline::new(config.gpu_acceleration);

        Self {
            config,
            timeline: TimelineManager::new(),
            assets: AssetLibrary::new(),
            effects: EffectsPipeline::new(),
            gpu,
        }
    }

    /// Initialize the editor (including GPU).
    pub fn initialize(&mut self) -> bool {
        self.gpu.initialize()
    }

    /// Get configuration.
    pub fn config(&self) -> &VideoEditorConfig {
        &self.config
    }

    /// Get timeline manager.
    pub fn timeline(&self) -> &TimelineManager {
        &self.timeline
    }

    /// Get mutable timeline manager.
    pub fn timeline_mut(&mut self) -> &mut TimelineManager {
        &mut self.timeline
    }

    /// Get asset library.
    pub fn assets(&self) -> &AssetLibrary {
        &self.assets
    }

    /// Get mutable asset library.
    pub fn assets_mut(&mut self) -> &mut AssetLibrary {
        &mut self.assets
    }

    /// Get effects pipeline.
    pub fn effects(&self) -> &EffectsPipeline {
        &self.effects
    }

    /// Get mutable effects pipeline.
    pub fn effects_mut(&mut self) -> &mut EffectsPipeline {
        &mut self.effects
    }

    /// Check if GPU is available.
    pub fn gpu_available(&self) -> bool {
        self.gpu.is_available()
    }

    /// Create a new project.
    pub fn new_project(&mut self) {
        self.timeline = TimelineManager::new();
        self.assets = AssetLibrary::new();
        self.effects = EffectsPipeline::new();

        // Add default tracks
        self.timeline.add_track("Video 1", TrackType::Video);
        self.timeline.add_track("Audio 1", TrackType::Audio);
    }
}

impl Default for VideoEditorPlugin {
    fn default() -> Self {
        Self::new(VideoEditorConfig::default())
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = VideoEditorPlugin::default();
        assert!(plugin.config().max_tracks > 0);
    }

    #[test]
    fn test_new_project() {
        let mut plugin = VideoEditorPlugin::default();
        plugin.new_project();
        assert_eq!(plugin.timeline().tracks().len(), 2);
    }

    #[test]
    fn test_asset_import() {
        let mut plugin = VideoEditorPlugin::default();
        let result = plugin.assets_mut().import_video("test.mp4");
        assert!(result.is_ok());
    }
}

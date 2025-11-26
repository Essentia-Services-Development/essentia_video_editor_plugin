//! # Essentia Video Editor Plugin
//!
//! AI-enhanced video editing with GPU acceleration for the Essentia platform.
//!
//! ## Features
//!
//! - Non-linear video editing
//! - GPU-accelerated effects and transitions
//! - AI-assisted content generation
//! - Timeline management
//! - Asset library integration
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Video Editor Plugin                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │  Timeline   │  │    Asset    │  │    Effects          │  │
//! │  │  Manager    │  │   Library   │  │    Pipeline         │  │
//! │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
//! │         │                │                     │             │
//! │         ▼                ▼                     ▼             │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │              GPU Rendering Pipeline                  │    │
//! │  │      (essentia_gpu_accel_kernel integration)         │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod types;
mod errors;
mod config;
mod timeline;
mod assets;
mod effects;
mod gpu_pipeline;
mod plugin;

pub use types::{
    VideoClip, AudioClip, TimelineTrack, TimelinePosition,
    VideoFormat, AudioFormat, Resolution, FrameRate,
};
pub use errors::{VideoEditorError, VideoEditorResult};
pub use config::VideoEditorConfig;
pub use timeline::TimelineManager;
pub use assets::AssetLibrary;
pub use effects::EffectsPipeline;
pub use gpu_pipeline::GpuPipeline;
pub use plugin::VideoEditorPlugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let config = VideoEditorConfig::default();
        assert!(config.max_tracks > 0);
    }
}

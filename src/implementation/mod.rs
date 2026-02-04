//! Video Editor Plugin implementations.
//!
//! This module contains all implementations for the Video Editor plugin:
//! - `VideoEditorConfig` - Configuration
//! - `AssetLibrary` - Asset management
//! - `EffectsPipeline` - Effects processing
//! - `GpuPipeline` - GPU-accelerated rendering
//! - `TimelineManager` - Timeline operations
//! - `VideoEditorPlugin` - Main plugin interface
//! - `TransitionManager` - Video transitions (GAP-220-B-001)
//! - `AudioMixer` - Audio mixing (GAP-220-B-002)
//! - `ExportQueue` - Export pipeline (GAP-220-B-003)
//! - `PreviewManager` - Preview system (GAP-220-B-004)
//! - `ColorGradingNode` - Color grading (GAP-220-B-005)
//! - `AnimationManager` - Keyframe animation (GAP-220-B-006)
//! - `MarkerManager` - Marker system (GAP-220-B-007)
//! - `ProjectManager` - Project management (GAP-220-B-008)

mod assets;
mod audio_mixer;
mod color_grading;
mod config;
mod effects;
mod export_pipeline;
mod gpu_pipeline;
mod keyframe_animation;
mod marker_system;
mod plugin;
mod preview_manager;
mod project_manager;
mod timeline;
mod transitions;

pub use assets::AssetLibrary;
pub use config::VideoEditorConfig;
pub use effects::{EffectType, EffectsPipeline, VideoEffect};
pub use gpu_pipeline::GpuPipeline;
pub use plugin::VideoEditorPlugin;
pub use timeline::TimelineManager;

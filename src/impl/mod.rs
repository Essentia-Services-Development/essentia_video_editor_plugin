//! Video Editor Plugin implementations.
//!
//! This module contains all implementations for the Video Editor plugin:
//! - `VideoEditorConfig` - Configuration
//! - `AssetLibrary` - Asset management
//! - `EffectsPipeline` - Effects processing
//! - `GpuPipeline` - GPU-accelerated rendering
//! - `TimelineManager` - Timeline operations
//! - `VideoEditorPlugin` - Main plugin interface

mod assets;
mod config;
mod effects;
mod gpu_pipeline;
mod plugin;
mod timeline;

pub use assets::AssetLibrary;
pub use config::VideoEditorConfig;
pub use effects::{EffectType, EffectsPipeline, VideoEffect};
pub use gpu_pipeline::GpuPipeline;
pub use plugin::VideoEditorPlugin;
pub use timeline::TimelineManager;

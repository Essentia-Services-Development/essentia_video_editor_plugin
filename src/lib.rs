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
//! - **Format conversion** (CR-015): Convert standard video/image/3D formats to
//!   FFUI
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
//! │         │                                                    │
//! │         ▼                                                    │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │           Format Converter (CR-015)                  │    │
//! │  │   MP4/MOV/MKV → EVLF  |  PNG/PSD → EFUI Layer        │    │
//! │  │   glTF/FBX → 3D Layer |  SVG → Vector Layer          │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod assets;
mod config;
pub mod converter;
mod effects;
mod errors;
pub mod evlf_types;
pub mod flexforge;
mod gpu_pipeline;
pub mod metadata;
mod plugin;
mod timeline;
mod types;

pub use assets::AssetLibrary;
pub use config::VideoEditorConfig;
pub use converter::{
    ConversionOptions, ConversionPhase, ConversionProgress, ConversionResult, ConversionStats,
    FormatConverter, InputFormat, InputFormatCategory, OutputFormat, ProgressCallback,
};
pub use effects::EffectsPipeline;
pub use errors::{VideoEditorError, VideoEditorResult};
pub use evlf_types::{
    BlendMode, BranchFork, BranchPoint, BranchType, EVLF_MAGIC, EVLF_VERSION, EvlfFlags,
    EvlfHeader, EvlfTrackHeader, EvlfTrackType, FrameIndexEntry, FrameType, TrackFlags,
};
pub use flexforge::VideoEditorFlexForge;
pub use gpu_pipeline::GpuPipeline;
pub use metadata::{
    Annotation, AnnotationType, BoundingBox, FrameMetadata, MetadataIndex, ObjectDetection,
    SceneClassification, SemanticRegion, TrackingState,
};
pub use plugin::VideoEditorPlugin;
pub use timeline::TimelineManager;
pub use types::{
    AudioClip, AudioFormat, FrameRate, Resolution, TimePosition, TimelinePosition, TimelineTrack,
    TrackType, VideoClip, VideoFormat,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let config = VideoEditorConfig::default();
        assert!(config.max_tracks > 0);
    }
}

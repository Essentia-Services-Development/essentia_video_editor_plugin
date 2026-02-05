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

#![allow(dead_code)]
//! - Asset library integration
//! - **Format conversion**: Convert standard video/image/3D formats to FFUI

// Video editor plugin pedantic lint allowances (VIDEO-LINT-STAGING-01)
// Video editing involves precision casts, builder patterns, complex types
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::unused_self)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::implicit_clone)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::if_not_else)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::float_cmp)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::match_bool)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::unnecessary_sort_by)]

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
//! │  │           Format Converter                  │    │
//! │  │   MP4/MOV/MKV → EVLF  |  PNG/PSD → EFUI Layer        │    │
//! │  │   glTF/FBX → 3D Layer |  SVG → Vector Layer          │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```

// EMD-compliant modules
pub mod errors;
mod implementation;
mod types;

// Domain modules (public APIs)
pub mod converter;
pub mod evlf_types;
pub mod flexforge;
pub mod metadata;

// Re-exports from errors
// Re-exports from converter
pub use converter::{
    ConversionOptions, ConversionPhase, ConversionProgress, ConversionResult, ConversionStats,
    FormatConverter, InputFormat, InputFormatCategory, OutputFormat, ProgressCallback,
};
pub use errors::{VideoEditorError, VideoEditorResult};
// Re-exports from evlf_types
pub use evlf_types::{
    BlendMode, BranchFork, BranchPoint, BranchType, EVLF_MAGIC, EVLF_VERSION, EvlfFlags,
    EvlfHeader, EvlfTrackHeader, EvlfTrackType, FrameIndexEntry, FrameType, TrackFlags,
};
// Re-exports from flexforge
pub use flexforge::VideoEditorFlexForge;
// Re-exports from impl
pub use implementation::{
    AssetLibrary, EffectType, EffectsPipeline, GpuPipeline, TimelineManager, VideoEditorConfig,
    VideoEditorPlugin, VideoEffect,
};
// Re-exports from metadata
pub use metadata::{
    Annotation, AnnotationType, BoundingBox, FrameMetadata, MetadataIndex, ObjectDetection,
    SceneClassification, SemanticRegion, TrackingState,
};
// Re-exports from types
pub use types::{
    AudioClip, AudioFormat, FrameRate, Resolution, TimePosition, TimelinePosition, TimelineTrack,
    TrackType, VideoClip, VideoFormat,
};

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let config = VideoEditorConfig::default();
        assert!(config.max_tracks > 0);
    }
}

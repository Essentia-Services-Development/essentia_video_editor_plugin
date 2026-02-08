//! Essentia Video Editor Plugin library.

#![allow(dead_code, missing_docs)]
#![allow(clippy::pedantic)]

pub mod errors;
mod implementation;
mod types;
pub mod converter;
pub mod evlf_types;
pub mod flexforge;
pub mod metadata;

pub use converter::{
    ConversionOptions, ConversionPhase, ConversionProgress, ConversionResult, ConversionStats,
    FormatConverter, InputFormat, InputFormatCategory, OutputFormat, ProgressCallback,
};
pub use errors::{VideoEditorError, VideoEditorResult};
pub use evlf_types::{
    BlendMode, BranchFork, BranchPoint, BranchType, EVLF_MAGIC, EVLF_VERSION, EvlfFlags,
    EvlfHeader, EvlfTrackHeader, EvlfTrackType, FrameIndexEntry, FrameType, TrackFlags,
};
pub use flexforge::VideoEditorFlexForge;
pub use implementation::{
    AssetLibrary, EffectType, EffectsPipeline, GpuPipeline, TimelineManager, VideoEditorConfig,
    VideoEditorPlugin, VideoEffect,
};
pub use metadata::{
    Annotation, AnnotationType, BoundingBox, FrameMetadata, MetadataIndex, ObjectDetection,
    SceneClassification, SemanticRegion, TrackingState,
};
pub use types::{
    AudioClip, AudioFormat, FrameRate, Resolution, TimePosition, TimelinePosition, TimelineTrack,
    TrackType, VideoClip, VideoFormat,
};

#[cfg(all(test, feature = "full-tests"))]
mod tests;

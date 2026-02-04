//! Video editor type definitions.
//!
//! Comprehensive types for NLE timeline operations, codec abstractions,
//! frame management, and professional video editing workflows.
//!
//! ## Architecture (Inspired by rust-av & media-rs)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Video Pipeline Types                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Core Types     │ Codec Types    │ Color Space   │ Pipeline     │
//! │  ─────────────  │ ─────────────  │ ─────────────  │ ─────────── │
//! │  Resolution     │ CodecId        │ ColorModel     │ GpuBackend  │
//! │  FrameRate      │ CodecParams    │ ColorPrimaries │ RenderPass  │
//! │  TimePosition   │ EncoderConfig  │ Formaton       │ RenderGraph │
//! │  TimeInfo       │ DecoderConfig  │ Chromaton      │ Pipeline    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Timeline       │ Clips          │ Frame Types                   │
//! │  ─────────────  │ ─────────────  │ ─────────────────────────────│
//! │  TimelineTrack  │ VideoClip      │ VideoFrame, AudioInfo         │
//! │  TimelineClip   │ AudioClip      │ FrameBuffer, FrameBufferPool  │
//! │  TrackType      │ ClipMetadata   │ FrameData, PlaneDescriptor    │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Note: Types are being built out for full NLE capabilities. Some types
//! may not be used immediately but provide the foundation for the complete
//! video editing pipeline.

// Allow dead code during development - types are being prepared for full NLE
#![allow(dead_code)]

pub mod clip;
pub mod codec;
pub mod color;
pub mod core;
pub mod frame;
pub mod pipeline;
pub mod timeline;

// Re-exports - Core types (primary API)
pub use core::{AudioFormat, FrameRate, Resolution, TimePosition, Timestamp, VideoFormat};

// Re-exports - Clip types (media clips)
pub use clip::{AudioClip, VideoClip};
// Re-exports - Timeline types (NLE operations)
pub use timeline::{TimelinePosition, TimelineTrack, TrackType};

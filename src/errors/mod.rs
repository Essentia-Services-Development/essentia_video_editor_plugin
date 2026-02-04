//! Error types for Video Editor Plugin.
//!
//! EMD-compliant error module providing `VideoEditorError` for video editing
//! operation failures.

mod video_editor_error;

pub use video_editor_error::{VideoEditorError, VideoEditorResult};

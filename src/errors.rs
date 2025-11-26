//! Video editor error types.

use core::fmt;

/// Video editor operation errors.
#[derive(Debug)]
pub enum VideoEditorError {
    /// Timeline error.
    Timeline(String),
    /// Asset error.
    Asset(String),
    /// Effect error.
    Effect(String),
    /// GPU error.
    Gpu(String),
    /// Export error.
    Export(String),
    /// IO error.
    Io(String),
}

impl fmt::Display for VideoEditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeline(msg) => write!(f, "Timeline error: {msg}"),
            Self::Asset(msg) => write!(f, "Asset error: {msg}"),
            Self::Effect(msg) => write!(f, "Effect error: {msg}"),
            Self::Gpu(msg) => write!(f, "GPU error: {msg}"),
            Self::Export(msg) => write!(f, "Export error: {msg}"),
            Self::Io(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

/// Result type for video editor operations.
pub type VideoEditorResult<T> = Result<T, VideoEditorError>;

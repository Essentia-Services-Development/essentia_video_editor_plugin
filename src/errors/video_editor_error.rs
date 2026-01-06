//! Video editor error definitions.
//!
//! Provides `VideoEditorError` for video editing operations including
//! timeline, asset, effect, GPU, export, format conversion, and decoding
//! errors.

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
    /// Unsupported format error.
    UnsupportedFormat(String),
    /// Conversion error.
    Conversion(String),
    /// Decoder error.
    Decoder(String),
}

impl VideoEditorError {
    /// Create an unsupported format error.
    #[must_use]
    pub fn unsupported_format(msg: impl Into<String>) -> Self {
        Self::UnsupportedFormat(msg.into())
    }

    /// Create a conversion error.
    #[must_use]
    pub fn conversion(msg: impl Into<String>) -> Self {
        Self::Conversion(msg.into())
    }

    /// Create a decoder error.
    #[must_use]
    pub fn decoder(msg: impl Into<String>) -> Self {
        Self::Decoder(msg.into())
    }
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
            Self::UnsupportedFormat(msg) => write!(f, "Unsupported format: {msg}"),
            Self::Conversion(msg) => write!(f, "Conversion error: {msg}"),
            Self::Decoder(msg) => write!(f, "Decoder error: {msg}"),
        }
    }
}

impl std::error::Error for VideoEditorError {}

/// Result type for video editor operations.
pub type VideoEditorResult<T> = Result<T, VideoEditorError>;

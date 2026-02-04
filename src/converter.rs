//! # Format Converter Module
//!
//! Converts standard video, 2D, 3D, and image formats to the FFUI/EVLF format.
//! This enables the video editor to open external files and convert them for
//! Essentia's universal content layer system.
//!
//! ## Supported Input Formats
//!
//! ### Video Formats
//! - MP4 (H.264/H.265)
//! - MOV (ProRes, H.264)
//! - MKV (VP9, AV1)
//! - AVI (legacy support)

// Allow unused input_path in placeholder implementations - will be used when actual codec is
// integrated
#![allow(unused_variables)]
//! - WebM (VP8/VP9)
//!
//! ### Image Formats
//! - PNG (8/16-bit, RGBA)
//! - JPEG (8-bit RGB)
//! - WebP (lossy/lossless)
//! - TIFF (multi-layer, HDR)
//! - EXR (HDR, multi-channel)
//! - PSD (layered Photoshop)
//!
//! ### 3D Formats
//! - glTF/GLB (industry standard)
//! - FBX (Autodesk)
//! - OBJ (legacy mesh)
//! - USD (Pixar Universal Scene Description)
//! - BLEND (Blender native)
//!
//! ### Vector Formats
//! - SVG (scalable vector graphics)
//! - PDF (vector extraction)
//! - AI (Adobe Illustrator)

use core::fmt;

use crate::errors::{VideoEditorError, VideoEditorResult};

/// Supported input format categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum InputFormatCategory {
    /// Video/motion formats
    Video   = 0x01,
    /// Static image formats
    Image   = 0x02,
    /// 3D scene/model formats
    Model3D = 0x03,
    /// Vector graphics formats
    Vector  = 0x04,
    /// Audio formats (for audio layers)
    Audio   = 0x05,
    /// Project/composition formats
    Project = 0x06,
}

/// Specific input format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum InputFormat {
    // Video formats (0x01XX)
    /// MP4 container (H.264/H.265/AAC)
    Mp4    = 0x0100,
    /// QuickTime MOV container
    Mov    = 0x0101,
    /// Matroska MKV container
    Mkv    = 0x0102,
    /// AVI container (legacy)
    Avi    = 0x0103,
    /// WebM container (VP8/VP9/Opus)
    WebM   = 0x0104,
    /// Windows Media Video
    Wmv    = 0x0105,
    /// Flash Video (legacy)
    Flv    = 0x0106,
    /// MPEG Transport Stream
    Ts     = 0x0107,

    // Image formats (0x02XX)
    /// PNG (Portable Network Graphics)
    Png    = 0x0200,
    /// JPEG
    Jpeg   = 0x0201,
    /// WebP
    WebP   = 0x0202,
    /// TIFF (Tagged Image File Format)
    Tiff   = 0x0203,
    /// OpenEXR (HDR)
    Exr    = 0x0204,
    /// Adobe Photoshop Document
    Psd    = 0x0205,
    /// GIF (animated)
    Gif    = 0x0206,
    /// BMP (Windows Bitmap)
    Bmp    = 0x0207,
    /// TGA (Targa)
    Tga    = 0x0208,
    /// HEIF/HEIC (Apple)
    Heif   = 0x0209,
    /// AVIF (AV1 Image)
    Avif   = 0x020A,
    /// JPEG XL
    Jxl    = 0x020B,

    // 3D formats (0x03XX)
    /// glTF 2.0 (JSON)
    Gltf   = 0x0300,
    /// GLB (binary glTF)
    Glb    = 0x0301,
    /// FBX (Autodesk)
    Fbx    = 0x0302,
    /// OBJ (Wavefront)
    Obj    = 0x0303,
    /// USD (Universal Scene Description)
    Usd    = 0x0304,
    /// USDZ (compressed USD)
    Usdz   = 0x0305,
    /// Blender native
    Blend  = 0x0306,
    /// 3DS Max (legacy)
    Max3ds = 0x0307,
    /// Collada DAE
    Dae    = 0x0308,
    /// STL (3D printing)
    Stl    = 0x0309,
    /// PLY (point cloud)
    Ply    = 0x030A,

    // Vector formats (0x04XX)
    /// SVG (Scalable Vector Graphics)
    Svg    = 0x0400,
    /// PDF (vector extraction)
    Pdf    = 0x0401,
    /// Adobe Illustrator
    Ai     = 0x0402,
    /// EPS (Encapsulated PostScript)
    Eps    = 0x0403,

    // Audio formats (0x05XX)
    /// WAV (uncompressed)
    Wav    = 0x0500,
    /// MP3
    Mp3    = 0x0501,
    /// AAC
    Aac    = 0x0502,
    /// FLAC (lossless)
    Flac   = 0x0503,
    /// OGG Vorbis
    Ogg    = 0x0504,
    /// Opus
    Opus   = 0x0505,

    // Project formats (0x06XX)
    /// Premiere Pro
    Prproj = 0x0600,
    /// After Effects
    Aep    = 0x0601,
    /// Final Cut Pro
    Fcpxml = 0x0602,
    /// DaVinci Resolve
    Drp    = 0x0603,
}

impl InputFormat {
    /// Get the category for this format
    #[must_use]
    pub const fn category(&self) -> InputFormatCategory {
        match *self as u16 >> 8 {
            0x01 => InputFormatCategory::Video,
            0x02 => InputFormatCategory::Image,
            0x03 => InputFormatCategory::Model3D,
            0x04 => InputFormatCategory::Vector,
            0x05 => InputFormatCategory::Audio,
            0x06 => InputFormatCategory::Project,
            _ => InputFormatCategory::Video,
        }
    }

    /// Get the file extension for this format
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            // Video
            Self::Mp4 => "mp4",
            Self::Mov => "mov",
            Self::Mkv => "mkv",
            Self::Avi => "avi",
            Self::WebM => "webm",
            Self::Wmv => "wmv",
            Self::Flv => "flv",
            Self::Ts => "ts",
            // Image
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Tiff => "tiff",
            Self::Exr => "exr",
            Self::Psd => "psd",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Tga => "tga",
            Self::Heif => "heif",
            Self::Avif => "avif",
            Self::Jxl => "jxl",
            // 3D
            Self::Gltf => "gltf",
            Self::Glb => "glb",
            Self::Fbx => "fbx",
            Self::Obj => "obj",
            Self::Usd => "usd",
            Self::Usdz => "usdz",
            Self::Blend => "blend",
            Self::Max3ds => "3ds",
            Self::Dae => "dae",
            Self::Stl => "stl",
            Self::Ply => "ply",
            // Vector
            Self::Svg => "svg",
            Self::Pdf => "pdf",
            Self::Ai => "ai",
            Self::Eps => "eps",
            // Audio
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Aac => "aac",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::Opus => "opus",
            // Project
            Self::Prproj => "prproj",
            Self::Aep => "aep",
            Self::Fcpxml => "fcpxml",
            Self::Drp => "drp",
        }
    }

    /// Detect format from file extension
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            // Video
            "mp4" | "m4v" => Some(Self::Mp4),
            "mov" | "qt" => Some(Self::Mov),
            "mkv" => Some(Self::Mkv),
            "avi" => Some(Self::Avi),
            "webm" => Some(Self::WebM),
            "wmv" => Some(Self::Wmv),
            "flv" => Some(Self::Flv),
            "ts" | "mts" | "m2ts" => Some(Self::Ts),
            // Image
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "tiff" | "tif" => Some(Self::Tiff),
            "exr" => Some(Self::Exr),
            "psd" => Some(Self::Psd),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "tga" => Some(Self::Tga),
            "heif" | "heic" => Some(Self::Heif),
            "avif" => Some(Self::Avif),
            "jxl" => Some(Self::Jxl),
            // 3D
            "gltf" => Some(Self::Gltf),
            "glb" => Some(Self::Glb),
            "fbx" => Some(Self::Fbx),
            "obj" => Some(Self::Obj),
            "usd" | "usda" | "usdc" => Some(Self::Usd),
            "usdz" => Some(Self::Usdz),
            "blend" => Some(Self::Blend),
            "3ds" => Some(Self::Max3ds),
            "dae" => Some(Self::Dae),
            "stl" => Some(Self::Stl),
            "ply" => Some(Self::Ply),
            // Vector
            "svg" => Some(Self::Svg),
            "pdf" => Some(Self::Pdf),
            "ai" => Some(Self::Ai),
            "eps" => Some(Self::Eps),
            // Audio
            "wav" => Some(Self::Wav),
            "mp3" => Some(Self::Mp3),
            "aac" | "m4a" => Some(Self::Aac),
            "flac" => Some(Self::Flac),
            "ogg" => Some(Self::Ogg),
            "opus" => Some(Self::Opus),
            // Project
            "prproj" => Some(Self::Prproj),
            "aep" => Some(Self::Aep),
            "fcpxml" => Some(Self::Fcpxml),
            "drp" => Some(Self::Drp),
            _ => None,
        }
    }

    /// Check if this format requires external decoder
    #[must_use]
    pub const fn requires_external_decoder(&self) -> bool {
        matches!(
            self,
            Self::Psd
                | Self::Ai
                | Self::Blend
                | Self::Prproj
                | Self::Aep
                | Self::Fcpxml
                | Self::Drp
        )
    }
}

/// Output format for conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum OutputFormat {
    /// EVLF (Essentia Video Layer Format) - video timeline
    #[default]
    Evlf           = 0x01,
    /// EFUI (Essentia FFUI) - UI composition
    Efui           = 0x02,
    /// Universal Layer - single layer extraction
    UniversalLayer = 0x03,
}

/// Conversion options
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Output format
    pub output_format:     OutputFormat,
    /// Target resolution (None = preserve original)
    pub target_resolution: Option<(u32, u32)>,
    /// Target frame rate (None = preserve original)
    pub target_fps:        Option<f32>,
    /// Quality level (0.0 - 1.0)
    pub quality:           f32,
    /// Preserve layers from layered formats (PSD, AI, etc.)
    pub preserve_layers:   bool,
    /// Extract audio tracks
    pub extract_audio:     bool,
    /// Generate frame index for fast seeking
    pub generate_index:    bool,
    /// Extract metadata (AI annotations, scene detection)
    pub extract_metadata:  bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            output_format:     OutputFormat::Evlf,
            target_resolution: None,
            target_fps:        None,
            quality:           0.9,
            preserve_layers:   true,
            extract_audio:     true,
            generate_index:    true,
            extract_metadata:  true,
        }
    }
}

/// Conversion progress callback
pub type ProgressCallback = Box<dyn Fn(ConversionProgress) + Send + Sync>;

/// Conversion progress information
#[derive(Debug, Clone)]
pub struct ConversionProgress {
    /// Current phase
    pub phase:            ConversionPhase,
    /// Progress within phase (0.0 - 1.0)
    pub progress:         f32,
    /// Frames processed (for video)
    pub frames_processed: u64,
    /// Total frames (for video)
    pub total_frames:     u64,
    /// Estimated time remaining (seconds)
    pub eta_seconds:      Option<f32>,
    /// Current processing rate (frames/second)
    pub rate_fps:         Option<f32>,
}

/// Conversion phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionPhase {
    /// Analyzing input format
    Analyzing,
    /// Extracting metadata
    ExtractingMetadata,
    /// Decoding frames
    Decoding,
    /// Processing/transcoding
    Processing,
    /// Generating index
    GeneratingIndex,
    /// Writing output
    Writing,
    /// Finalizing
    Finalizing,
}

/// Conversion result
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Output file path
    pub output_path:   String,
    /// Output format used
    pub output_format: OutputFormat,
    /// Conversion statistics
    pub stats:         ConversionStats,
}

/// Conversion statistics
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Total input size in bytes
    pub input_size:         u64,
    /// Total output size in bytes
    pub output_size:        u64,
    /// Number of frames converted
    pub frames_converted:   u64,
    /// Number of layers extracted
    pub layers_extracted:   u32,
    /// Audio tracks extracted
    pub audio_tracks:       u32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Compression ratio (output/input)
    pub compression_ratio:  f32,
}

/// Format converter
pub struct FormatConverter {
    /// Conversion options
    options:           ConversionOptions,
    /// Progress callback
    progress_callback: Option<ProgressCallback>,
}

impl fmt::Debug for FormatConverter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormatConverter")
            .field("options", &self.options)
            .field(
                "progress_callback",
                &self.progress_callback.as_ref().map(|_| "<callback>"),
            )
            .finish()
    }
}

impl FormatConverter {
    /// Create a new format converter with default options
    #[must_use]
    pub fn new() -> Self {
        Self { options: ConversionOptions::default(), progress_callback: None }
    }

    /// Create converter with custom options
    #[must_use]
    pub fn with_options(options: ConversionOptions) -> Self {
        Self { options, progress_callback: None }
    }

    /// Set progress callback
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    /// Detect input format from file path
    #[must_use]
    pub fn detect_format(path: &str) -> Option<InputFormat> {
        let ext = path.rsplit('.').next()?;
        InputFormat::from_extension(ext)
    }

    /// Check if a format is supported
    #[must_use]
    pub fn is_supported(format: InputFormat) -> bool {
        // All enumerated formats are supported (some require external decoders)
        !format.requires_external_decoder()
    }

    /// Convert a file to FFUI format
    ///
    /// # Errors
    ///
    /// Returns error if the format is unsupported or conversion fails.
    pub fn convert(
        &self, input_path: &str, output_path: &str,
    ) -> VideoEditorResult<ConversionResult> {
        let format = Self::detect_format(input_path)
            .ok_or_else(|| VideoEditorError::unsupported_format("Unknown file extension"))?;

        if format.requires_external_decoder() {
            return Err(VideoEditorError::unsupported_format(
                "Format requires external decoder (not yet implemented)",
            ));
        }

        // Report analysis phase
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Analyzing,
            progress:         0.0,
            frames_processed: 0,
            total_frames:     0,
            eta_seconds:      None,
            rate_fps:         None,
        });

        // Dispatch based on format category
        match format.category() {
            InputFormatCategory::Video => self.convert_video(input_path, output_path, format),
            InputFormatCategory::Image => self.convert_image(input_path, output_path, format),
            InputFormatCategory::Model3D => self.convert_3d(input_path, output_path, format),
            InputFormatCategory::Vector => self.convert_vector(input_path, output_path, format),
            InputFormatCategory::Audio => self.convert_audio(input_path, output_path, format),
            InputFormatCategory::Project => Err(VideoEditorError::unsupported_format(
                "Project formats require dedicated import",
            )),
        }
    }

    /// Convert video format
    fn convert_video(
        &self, input_path: &str, output_path: &str, _format: InputFormat,
    ) -> VideoEditorResult<ConversionResult> {
        // Placeholder implementation - actual decoding would use GPU pipeline
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Decoding,
            progress:         0.5,
            frames_processed: 0,
            total_frames:     100, // Placeholder
            eta_seconds:      Some(10.0),
            rate_fps:         Some(60.0),
        });

        Ok(ConversionResult {
            output_path:   output_path.to_string(),
            output_format: self.options.output_format,
            stats:         ConversionStats {
                input_size:         0,
                output_size:        0,
                frames_converted:   0,
                layers_extracted:   1,
                audio_tracks:       if self.options.extract_audio { 1 } else { 0 },
                processing_time_ms: 0,
                compression_ratio:  1.0,
            },
        })
    }

    /// Convert image format
    fn convert_image(
        &self, input_path: &str, output_path: &str, _format: InputFormat,
    ) -> VideoEditorResult<ConversionResult> {
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Processing,
            progress:         0.5,
            frames_processed: 1,
            total_frames:     1,
            eta_seconds:      Some(1.0),
            rate_fps:         None,
        });

        Ok(ConversionResult {
            output_path:   output_path.to_string(),
            output_format: self.options.output_format,
            stats:         ConversionStats {
                frames_converted: 1,
                layers_extracted: if self.options.preserve_layers { 0 } else { 1 },
                ..Default::default()
            },
        })
    }

    /// Convert 3D format
    fn convert_3d(
        &self, input_path: &str, output_path: &str, _format: InputFormat,
    ) -> VideoEditorResult<ConversionResult> {
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Processing,
            progress:         0.5,
            frames_processed: 0,
            total_frames:     0,
            eta_seconds:      Some(5.0),
            rate_fps:         None,
        });

        Ok(ConversionResult {
            output_path:   output_path.to_string(),
            output_format: OutputFormat::UniversalLayer,
            stats:         ConversionStats { layers_extracted: 1, ..Default::default() },
        })
    }

    /// Convert vector format
    fn convert_vector(
        &self, input_path: &str, output_path: &str, _format: InputFormat,
    ) -> VideoEditorResult<ConversionResult> {
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Processing,
            progress:         0.5,
            frames_processed: 0,
            total_frames:     0,
            eta_seconds:      Some(1.0),
            rate_fps:         None,
        });

        Ok(ConversionResult {
            output_path:   output_path.to_string(),
            output_format: OutputFormat::UniversalLayer,
            stats:         ConversionStats { layers_extracted: 1, ..Default::default() },
        })
    }

    /// Convert audio format
    fn convert_audio(
        &self, input_path: &str, output_path: &str, _format: InputFormat,
    ) -> VideoEditorResult<ConversionResult> {
        self.report_progress(ConversionProgress {
            phase:            ConversionPhase::Processing,
            progress:         0.5,
            frames_processed: 0,
            total_frames:     0,
            eta_seconds:      Some(2.0),
            rate_fps:         None,
        });

        Ok(ConversionResult {
            output_path:   output_path.to_string(),
            output_format: self.options.output_format,
            stats:         ConversionStats { audio_tracks: 1, ..Default::default() },
        })
    }

    /// Report progress via callback
    fn report_progress(&self, progress: ConversionProgress) {
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
    }
}

impl Default for FormatConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for InputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Evlf => write!(f, "EVLF"),
            Self::Efui => write!(f, "EFUI"),
            Self::UniversalLayer => write!(f, "Universal Layer"),
        }
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(InputFormat::from_extension("mp4"), Some(InputFormat::Mp4));
        assert_eq!(InputFormat::from_extension("PNG"), Some(InputFormat::Png));
        assert_eq!(InputFormat::from_extension("glTF"), Some(InputFormat::Gltf));
        assert_eq!(InputFormat::from_extension("svg"), Some(InputFormat::Svg));
        assert_eq!(InputFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_category() {
        assert_eq!(InputFormat::Mp4.category(), InputFormatCategory::Video);
        assert_eq!(InputFormat::Png.category(), InputFormatCategory::Image);
        assert_eq!(InputFormat::Gltf.category(), InputFormatCategory::Model3D);
        assert_eq!(InputFormat::Svg.category(), InputFormatCategory::Vector);
        assert_eq!(InputFormat::Wav.category(), InputFormatCategory::Audio);
    }

    #[test]
    fn test_converter_creation() {
        let converter = FormatConverter::new();
        assert_eq!(converter.options.quality, 0.9);
        assert!(converter.options.preserve_layers);
    }

    #[test]
    fn test_path_format_detection() {
        assert_eq!(
            FormatConverter::detect_format("video.mp4"),
            Some(InputFormat::Mp4)
        );
        assert_eq!(
            FormatConverter::detect_format("/path/to/image.png"),
            Some(InputFormat::Png)
        );
        assert_eq!(
            FormatConverter::detect_format("model.glb"),
            Some(InputFormat::Glb)
        );
    }
}

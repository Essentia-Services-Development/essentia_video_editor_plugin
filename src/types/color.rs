//! Color space and color management types.
//!
//! Provides ACES-compliant color pipeline support with HDR capabilities.
//! Inspired by rust-av's Formaton and media-rs color management.

use std::sync::Arc;

/// Color model enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorModel {
    /// RGB color model.
    #[default]
    Rgb,
    /// YUV/YCbCr color model.
    Yuv,
    /// YCoCg color model.
    YCoCg,
    /// XYZ color model (CIE 1931).
    Xyz,
    /// Lab color model.
    Lab,
    /// HSV color model.
    Hsv,
    /// CMYK color model.
    Cmyk,
    /// Grayscale.
    Gray,
}

/// Color primaries (chromaticity coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorPrimaries {
    /// ITU-R BT.709 (sRGB, Rec.709).
    #[default]
    Bt709,
    /// ITU-R BT.601 PAL.
    Bt601Pal,
    /// ITU-R BT.601 NTSC.
    Bt601Ntsc,
    /// ITU-R BT.2020 (UHD/HDR).
    Bt2020,
    /// DCI-P3 (digital cinema).
    DciP3,
    /// Display P3 (Apple).
    DisplayP3,
    /// Adobe RGB.
    AdobeRgb,
    /// ACES AP0 (primaries).
    AcesAp0,
    /// ACES AP1 (working space).
    AcesAp1,
    /// Custom/unknown primaries.
    Unknown,
}

impl ColorPrimaries {
    /// Returns whether this is an HDR-capable color space.
    #[must_use]
    pub const fn is_hdr_capable(&self) -> bool {
        matches!(
            self,
            Self::Bt2020 | Self::DciP3 | Self::DisplayP3 | Self::AcesAp0 | Self::AcesAp1
        )
    }

    /// Returns the gamut coverage relative to human vision (approximate).
    #[must_use]
    pub const fn gamut_coverage_percent(&self) -> u8 {
        match self {
            Self::Bt709 => 36,
            Self::Bt601Pal | Self::Bt601Ntsc => 35,
            Self::Bt2020 => 76,
            Self::DciP3 => 54,
            Self::DisplayP3 => 54,
            Self::AdobeRgb => 52,
            Self::AcesAp0 => 100,
            Self::AcesAp1 => 53,
            Self::Unknown => 0,
        }
    }
}

/// Transfer function (gamma/OETF/EOTF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransferFunction {
    /// Linear (no transfer function).
    Linear,
    /// sRGB transfer function.
    #[default]
    Srgb,
    /// ITU-R BT.709 (approximately gamma 2.4).
    Bt709,
    /// ITU-R BT.2020 10-bit.
    Bt202010,
    /// ITU-R BT.2020 12-bit.
    Bt202012,
    /// PQ (Perceptual Quantizer) - SMPTE ST 2084.
    Pq,
    /// HLG (Hybrid Log-Gamma) - ARIB STD-B67.
    Hlg,
    /// Gamma 2.2.
    Gamma22,
    /// Gamma 2.4.
    Gamma24,
    /// Gamma 2.6 (DCI).
    Gamma26,
    /// Log (S-Log3, LogC, etc.).
    Log,
    /// Custom/unknown transfer.
    Unknown,
}

impl TransferFunction {
    /// Returns whether this is an HDR transfer function.
    #[must_use]
    pub const fn is_hdr(&self) -> bool {
        matches!(self, Self::Pq | Self::Hlg)
    }

    /// Returns the nominal peak luminance in nits.
    #[must_use]
    pub const fn peak_luminance_nits(&self) -> u32 {
        match self {
            Self::Linear | Self::Srgb | Self::Bt709 | Self::Gamma22 | Self::Gamma24 => 100,
            Self::Bt202010 | Self::Bt202012 => 100,
            Self::Pq => 10000,
            Self::Hlg => 1000,
            Self::Gamma26 => 48, // DCI reference
            Self::Log => 100,    // Varies by implementation
            Self::Unknown => 100,
        }
    }
}

/// Matrix coefficients for YUV/RGB conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MatrixCoefficients {
    /// Identity (RGB).
    Identity,
    /// ITU-R BT.709.
    #[default]
    Bt709,
    /// ITU-R BT.601.
    Bt601,
    /// ITU-R BT.2020 non-constant luminance.
    Bt2020Ncl,
    /// ITU-R BT.2020 constant luminance.
    Bt2020Cl,
    /// SMPTE ST 2085 (ICTCP).
    ICtCp,
    /// Unknown.
    Unknown,
}

/// Chroma location for subsampled formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChromaLocation {
    /// Left-aligned chroma samples.
    #[default]
    Left,
    /// Center-aligned chroma samples.
    Center,
    /// Top-left aligned.
    TopLeft,
    /// Top aligned.
    Top,
    /// Bottom-left aligned.
    BottomLeft,
    /// Bottom aligned.
    Bottom,
}

/// Color range (full vs limited).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorRange {
    /// Full range (0-255 for 8-bit).
    Full,
    /// Limited/TV range (16-235 for 8-bit luma).
    #[default]
    Limited,
}

/// Component/channel descriptor (inspired by rust-av's Chromaton).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chromaton {
    /// Horizontal subsampling shift (0 = no subsampling).
    h_ss:   u8,
    /// Vertical subsampling shift (0 = no subsampling).
    v_ss:   u8,
    /// Whether the component is packed with others.
    packed: bool,
    /// Bit depth for this component.
    depth:  u8,
    /// Byte offset within packed pixel (0 for planar).
    offset: u8,
    /// Component identifier (Y, U, V, R, G, B, A).
    id:     ComponentId,
}

impl Chromaton {
    /// Creates a new chromaton with full resolution.
    #[must_use]
    pub const fn new(id: ComponentId, depth: u8) -> Self {
        Self { h_ss: 0, v_ss: 0, packed: false, depth, offset: 0, id }
    }

    /// Creates a new chromaton with subsampling.
    #[must_use]
    pub const fn with_subsampling(id: ComponentId, depth: u8, h_ss: u8, v_ss: u8) -> Self {
        Self { h_ss, v_ss, packed: false, depth, offset: 0, id }
    }

    /// Creates a packed chromaton.
    #[must_use]
    pub const fn packed(id: ComponentId, depth: u8, offset: u8) -> Self {
        Self { h_ss: 0, v_ss: 0, packed: true, depth, offset, id }
    }

    /// Returns the subsampling factors as (horizontal, vertical).
    #[must_use]
    pub const fn subsampling(&self) -> (u8, u8) {
        (self.h_ss, self.v_ss)
    }

    /// Returns the bit depth.
    #[must_use]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// Returns whether this component is packed.
    #[must_use]
    pub const fn is_packed(&self) -> bool {
        self.packed
    }

    /// Returns the component identifier.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.id
    }
}

/// Component identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentId {
    /// Luma (Y).
    Y,
    /// Chroma blue (U/Cb).
    U,
    /// Chroma red (V/Cr).
    V,
    /// Red.
    R,
    /// Green.
    G,
    /// Blue.
    B,
    /// Alpha.
    A,
}

/// Complete pixel format descriptor (inspired by rust-av's Formaton).
#[derive(Debug, Clone, PartialEq)]
pub struct Formaton {
    /// Color model.
    model:      ColorModel,
    /// Color primaries.
    primaries:  ColorPrimaries,
    /// Transfer function.
    transfer:   TransferFunction,
    /// Matrix coefficients.
    matrix:     MatrixCoefficients,
    /// Chroma location.
    chroma_loc: ChromaLocation,
    /// Color range.
    range:      ColorRange,
    /// Component descriptors.
    components: Vec<Chromaton>,
    /// Format name/identifier.
    name:       String,
    /// Whether format has alpha channel.
    has_alpha:  bool,
    /// Palette for indexed formats.
    palette:    bool,
}

impl Formaton {
    /// Creates a new format descriptor.
    #[must_use]
    pub fn new(name: impl Into<String>, model: ColorModel, components: Vec<Chromaton>) -> Self {
        let has_alpha = components.iter().any(|c| c.id == ComponentId::A);
        Self {
            model,
            primaries: ColorPrimaries::default(),
            transfer: TransferFunction::default(),
            matrix: MatrixCoefficients::default(),
            chroma_loc: ChromaLocation::default(),
            range: ColorRange::default(),
            components,
            name: name.into(),
            has_alpha,
            palette: false,
        }
    }

    /// Returns the number of components.
    #[must_use]
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Returns a specific component descriptor.
    #[must_use]
    pub fn component(&self, index: usize) -> Option<&Chromaton> {
        self.components.get(index)
    }

    /// Returns total bit depth across all components.
    #[must_use]
    pub fn total_depth(&self) -> u8 {
        self.components.iter().map(|c| c.depth).sum()
    }

    /// Returns whether format has alpha.
    #[must_use]
    pub const fn has_alpha(&self) -> bool {
        self.has_alpha
    }

    /// Returns the color model.
    #[must_use]
    pub const fn model(&self) -> ColorModel {
        self.model
    }

    /// Returns the color primaries.
    #[must_use]
    pub const fn primaries(&self) -> ColorPrimaries {
        self.primaries
    }

    /// Returns the transfer function.
    #[must_use]
    pub const fn transfer(&self) -> TransferFunction {
        self.transfer
    }

    /// Creates RGB24 format.
    #[must_use]
    pub fn rgb24() -> Arc<Self> {
        Arc::new(Self::new("RGB24", ColorModel::Rgb, vec![
            Chromaton::new(ComponentId::R, 8),
            Chromaton::new(ComponentId::G, 8),
            Chromaton::new(ComponentId::B, 8),
        ]))
    }

    /// Creates RGBA32 format.
    #[must_use]
    pub fn rgba32() -> Arc<Self> {
        Arc::new(Self::new("RGBA32", ColorModel::Rgb, vec![
            Chromaton::new(ComponentId::R, 8),
            Chromaton::new(ComponentId::G, 8),
            Chromaton::new(ComponentId::B, 8),
            Chromaton::new(ComponentId::A, 8),
        ]))
    }

    /// Creates YUV420P format (planar 4:2:0).
    #[must_use]
    pub fn yuv420p() -> Arc<Self> {
        Arc::new(Self::new("YUV420P", ColorModel::Yuv, vec![
            Chromaton::new(ComponentId::Y, 8),
            Chromaton::with_subsampling(ComponentId::U, 8, 1, 1),
            Chromaton::with_subsampling(ComponentId::V, 8, 1, 1),
        ]))
    }

    /// Creates YUV422P format (planar 4:2:2).
    #[must_use]
    pub fn yuv422p() -> Arc<Self> {
        Arc::new(Self::new("YUV422P", ColorModel::Yuv, vec![
            Chromaton::new(ComponentId::Y, 8),
            Chromaton::with_subsampling(ComponentId::U, 8, 1, 0),
            Chromaton::with_subsampling(ComponentId::V, 8, 1, 0),
        ]))
    }

    /// Creates YUV444P format (planar 4:4:4).
    #[must_use]
    pub fn yuv444p() -> Arc<Self> {
        Arc::new(Self::new("YUV444P", ColorModel::Yuv, vec![
            Chromaton::new(ComponentId::Y, 8),
            Chromaton::new(ComponentId::U, 8),
            Chromaton::new(ComponentId::V, 8),
        ]))
    }

    /// Creates YUV420P10 format (10-bit 4:2:0).
    #[must_use]
    pub fn yuv420p10() -> Arc<Self> {
        Arc::new(Self::new("YUV420P10", ColorModel::Yuv, vec![
            Chromaton::new(ComponentId::Y, 10),
            Chromaton::with_subsampling(ComponentId::U, 10, 1, 1),
            Chromaton::with_subsampling(ComponentId::V, 10, 1, 1),
        ]))
    }
}

impl Default for Formaton {
    fn default() -> Self {
        Self::new("Unknown", ColorModel::Rgb, vec![])
    }
}

/// HDR metadata container.
#[derive(Debug, Clone, Default)]
pub struct HdrMetadata {
    /// Maximum content light level (nits).
    pub max_cll:             Option<u32>,
    /// Maximum frame-average light level (nits).
    pub max_fall:            Option<u32>,
    /// Mastering display luminance (min, max in nits).
    pub mastering_luminance: Option<(f32, f32)>,
    /// Mastering display primaries (rx, ry, gx, gy, bx, by).
    pub mastering_primaries: Option<[f32; 6]>,
    /// White point (x, y).
    pub white_point:         Option<(f32, f32)>,
}

impl HdrMetadata {
    /// Creates HDR10 metadata.
    #[must_use]
    pub fn hdr10(max_cll: u32, max_fall: u32) -> Self {
        Self {
            max_cll:             Some(max_cll),
            max_fall:            Some(max_fall),
            mastering_luminance: Some((0.0001, 1000.0)),
            mastering_primaries: Some([0.68, 0.32, 0.265, 0.69, 0.15, 0.06]), // BT.2020
            white_point:         Some((0.3127, 0.329)),                       // D65
        }
    }

    /// Returns whether this contains valid HDR metadata.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.max_cll.is_some() || self.max_fall.is_some()
    }
}

/// Color correction parameters.
#[derive(Debug, Clone)]
pub struct ColorCorrection {
    /// Exposure adjustment (-5.0 to +5.0 stops).
    pub exposure:    f32,
    /// Contrast (0.0 to 2.0).
    pub contrast:    f32,
    /// Saturation (0.0 to 2.0).
    pub saturation:  f32,
    /// Shadows lift (RGB).
    pub shadows:     [f32; 3],
    /// Midtones gamma (RGB).
    pub midtones:    [f32; 3],
    /// Highlights gain (RGB).
    pub highlights:  [f32; 3],
    /// Temperature (-100 to +100).
    pub temperature: f32,
    /// Tint (-100 to +100).
    pub tint:        f32,
}

impl Default for ColorCorrection {
    fn default() -> Self {
        Self {
            exposure:    0.0,
            contrast:    1.0,
            saturation:  1.0,
            shadows:     [0.0, 0.0, 0.0],
            midtones:    [1.0, 1.0, 1.0],
            highlights:  [1.0, 1.0, 1.0],
            temperature: 0.0,
            tint:        0.0,
        }
    }
}

impl ColorCorrection {
    /// Returns whether this is a neutral (no-op) correction.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        (self.exposure - 0.0).abs() < f32::EPSILON
            && (self.contrast - 1.0).abs() < f32::EPSILON
            && (self.saturation - 1.0).abs() < f32::EPSILON
            && self.shadows == [0.0, 0.0, 0.0]
            && self.midtones == [1.0, 1.0, 1.0]
            && self.highlights == [1.0, 1.0, 1.0]
            && (self.temperature - 0.0).abs() < f32::EPSILON
            && (self.tint - 0.0).abs() < f32::EPSILON
    }
}

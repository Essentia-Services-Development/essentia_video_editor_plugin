//! Color Grading for Essentia Video Editor Plugin
//! GAP-220-B-005: Professional Color Grading System
//!
//! Features: LUT support, color wheels, curves, scopes,
//! HSL adjustment, color matching, and node-based grading.

use essentia_color_types::{Color, Hsl};

/// Color space for grading operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorSpace {
    /// Standard sRGB.
    #[default]
    Srgb,
    /// Rec. 709 (HD broadcast).
    Rec709,
    /// Rec. 2020 (UHD/HDR).
    Rec2020,
    /// DCI-P3 (digital cinema).
    DciP3,
    /// ACEScg (VFX working space).
    AcesCg,
    /// ACES 2065-1 (archival).
    Aces2065,
    /// Log encoding (generic).
    Log,
    /// S-Log3 (Sony).
    SLog3,
    /// V-Log (Panasonic).
    VLog,
    /// C-Log (Canon).
    CLog,
    /// ProRes Log.
    ProResLog,
}

impl ColorSpace {
    /// Returns whether this is a log color space.
    #[must_use]
    pub const fn is_log(&self) -> bool {
        matches!(
            self,
            Self::Log | Self::SLog3 | Self::VLog | Self::CLog | Self::ProResLog
        )
    }

    /// Returns the gamma value for this space.
    #[must_use]
    pub const fn gamma(&self) -> f32 {
        match self {
            Self::Srgb => 2.2,
            Self::Rec709 | Self::Rec2020 => 2.4,
            _ => 1.0,
        }
    }
}

/// Color wheel adjustment (shadows/midtones/highlights).
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorWheel {
    /// Hue rotation (-1.0 to 1.0).
    pub hue:        f32,
    /// Saturation (-1.0 to 1.0, where 0 = no change).
    pub saturation: f32,
    /// Brightness offset (-1.0 to 1.0).
    pub brightness: f32,
    /// Master offset (color tint).
    pub offset:     Color,
}

impl ColorWheel {
    /// Creates a neutral color wheel.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            hue:        0.0,
            saturation: 0.0,
            brightness: 0.0,
            offset:     Color::rgb(0.0, 0.0, 0.0),
        }
    }

    /// Applies the color wheel to a color.
    #[must_use]
    pub fn apply(&self, color: &Color) -> Color {
        let hsl = color.to_hsl();
        let (h, s, l) = (hsl.h, hsl.s, hsl.l);

        // Apply hue rotation
        let new_h = (h + self.hue).rem_euclid(1.0);

        // Apply saturation
        let new_s = (s * (1.0 + self.saturation)).clamp(0.0, 1.0);

        // Apply brightness
        let new_l = (l + self.brightness).clamp(0.0, 1.0);

        let mut result = Hsl::new(new_h, new_s, new_l).to_rgb();

        // Apply offset
        result.r += self.offset.r;
        result.g += self.offset.g;
        result.b += self.offset.b;
        result.a = color.a;

        result
    }
}

/// Three-way color corrector (shadows, midtones, highlights).
#[derive(Debug, Clone, Default)]
pub struct ThreeWayCorrector {
    /// Shadows adjustment.
    pub shadows:         ColorWheel,
    /// Midtones adjustment.
    pub midtones:        ColorWheel,
    /// Highlights adjustment.
    pub highlights:      ColorWheel,
    /// Shadow range (0.0 to 1.0).
    pub shadow_range:    f32,
    /// Highlight range (0.0 to 1.0).
    pub highlight_range: f32,
}

impl ThreeWayCorrector {
    /// Creates a neutral corrector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shadows:         ColorWheel::neutral(),
            midtones:        ColorWheel::neutral(),
            highlights:      ColorWheel::neutral(),
            shadow_range:    0.25,
            highlight_range: 0.75,
        }
    }

    /// Calculates the weight for shadows at given luminance.
    fn shadow_weight(&self, lum: f32) -> f32 {
        if lum < self.shadow_range {
            1.0
        } else if lum < self.shadow_range + 0.1 {
            1.0 - (lum - self.shadow_range) / 0.1
        } else {
            0.0
        }
    }

    /// Calculates the weight for highlights at given luminance.
    fn highlight_weight(&self, lum: f32) -> f32 {
        if lum > self.highlight_range {
            1.0
        } else if lum > self.highlight_range - 0.1 {
            (lum - (self.highlight_range - 0.1)) / 0.1
        } else {
            0.0
        }
    }

    /// Applies the three-way correction.
    #[must_use]
    pub fn apply(&self, color: &Color) -> Color {
        let lum = color.luminance();
        let sw = self.shadow_weight(lum);
        let hw = self.highlight_weight(lum);
        let mw = 1.0 - sw - hw;

        let shadow_color = self.shadows.apply(color);
        let mid_color = self.midtones.apply(color);
        let highlight_color = self.highlights.apply(color);

        Color::new(
            shadow_color.r * sw + mid_color.r * mw + highlight_color.r * hw,
            shadow_color.g * sw + mid_color.g * mw + highlight_color.g * hw,
            shadow_color.b * sw + mid_color.b * mw + highlight_color.b * hw,
            color.a,
        )
    }
}

/// Curve point for color curves.
#[derive(Debug, Clone, Copy, Default)]
pub struct CurvePoint {
    /// Input value (0.0 to 1.0).
    pub x: f32,
    /// Output value (0.0 to 1.0).
    pub y: f32,
}

impl CurvePoint {
    /// Creates a new curve point.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Color curve with control points.
#[derive(Debug, Clone)]
pub struct ColorCurve {
    /// Control points (sorted by x).
    points: Vec<CurvePoint>,
    /// Precomputed LUT for fast lookup.
    lut:    Vec<f32>,
}

impl ColorCurve {
    /// Creates a linear (identity) curve.
    #[must_use]
    pub fn linear() -> Self {
        let points = vec![CurvePoint::new(0.0, 0.0), CurvePoint::new(1.0, 1.0)];
        let mut curve = Self { points, lut: Vec::new() };
        curve.rebuild_lut();
        curve
    }

    /// Creates an S-curve for contrast.
    #[must_use]
    pub fn s_curve(strength: f32) -> Self {
        let s = strength.clamp(0.0, 1.0);
        let points = vec![
            CurvePoint::new(0.0, 0.0),
            CurvePoint::new(0.25, 0.25 - 0.1 * s),
            CurvePoint::new(0.5, 0.5),
            CurvePoint::new(0.75, 0.75 + 0.1 * s),
            CurvePoint::new(1.0, 1.0),
        ];
        let mut curve = Self { points, lut: Vec::new() };
        curve.rebuild_lut();
        curve
    }

    /// Adds a control point.
    pub fn add_point(&mut self, x: f32, y: f32) {
        let point = CurvePoint::new(x.clamp(0.0, 1.0), y.clamp(0.0, 1.0));

        // Insert in sorted order
        let pos = self.points.iter().position(|p| p.x > point.x).unwrap_or(self.points.len());
        self.points.insert(pos, point);
        self.rebuild_lut();
    }

    /// Removes the nearest point to x.
    pub fn remove_point(&mut self, x: f32) -> bool {
        if self.points.len() <= 2 {
            return false; // Keep at least 2 points
        }

        if let Some(pos) = self
            .points
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.x - x)
                    .abs()
                    .partial_cmp(&(b.x - x).abs())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
        {
            self.points.remove(pos);
            self.rebuild_lut();
            true
        } else {
            false
        }
    }

    /// Rebuilds the lookup table.
    fn rebuild_lut(&mut self) {
        const LUT_SIZE: usize = 256;
        let mut new_lut = vec![0.0; LUT_SIZE];

        for (i, value) in new_lut.iter_mut().enumerate().take(LUT_SIZE) {
            let x = i as f32 / (LUT_SIZE - 1) as f32;
            *value = self.evaluate_spline(x);
        }

        self.lut = new_lut;
    }

    /// Evaluates the curve using cubic interpolation.
    fn evaluate_spline(&self, x: f32) -> f32 {
        if self.points.is_empty() {
            return x;
        }

        if x <= self.points[0].x {
            return self.points[0].y;
        }

        if x >= self.points[self.points.len() - 1].x {
            return self.points[self.points.len() - 1].y;
        }

        // Find segment
        let mut i = 0;
        while i < self.points.len() - 1 && self.points[i + 1].x < x {
            i += 1;
        }

        let p0 = &self.points[i];
        let p1 = &self.points[i + 1];

        // Linear interpolation (could be upgraded to cubic spline)
        let t = (x - p0.x) / (p1.x - p0.x);
        p0.y + t * (p1.y - p0.y)
    }

    /// Evaluates the curve at x using the LUT.
    #[must_use]
    pub fn evaluate(&self, x: f32) -> f32 {
        if self.lut.is_empty() {
            return x;
        }

        let x = x.clamp(0.0, 1.0);
        let idx = (x * (self.lut.len() - 1) as f32) as usize;
        self.lut[idx.min(self.lut.len() - 1)]
    }

    /// Returns the control points.
    #[must_use]
    pub fn points(&self) -> &[CurvePoint] {
        &self.points
    }
}

impl Default for ColorCurve {
    fn default() -> Self {
        Self::linear()
    }
}

/// RGBA curves for color correction.
#[derive(Debug, Clone, Default)]
pub struct ColorCurves {
    /// Master (luminance) curve.
    pub master: ColorCurve,
    /// Red channel curve.
    pub red:    ColorCurve,
    /// Green channel curve.
    pub green:  ColorCurve,
    /// Blue channel curve.
    pub blue:   ColorCurve,
}

impl ColorCurves {
    /// Creates neutral curves.
    #[must_use]
    pub fn new() -> Self {
        Self {
            master: ColorCurve::linear(),
            red:    ColorCurve::linear(),
            green:  ColorCurve::linear(),
            blue:   ColorCurve::linear(),
        }
    }

    /// Applies curves to a color.
    #[must_use]
    pub fn apply(&self, color: &Color) -> Color {
        // Apply per-channel curves
        let r = self.red.evaluate(color.r);
        let g = self.green.evaluate(color.g);
        let b = self.blue.evaluate(color.b);

        // Apply master curve to result
        Color::new(
            self.master.evaluate(r),
            self.master.evaluate(g),
            self.master.evaluate(b),
            color.a,
        )
    }
}

/// LUT (Look-Up Table) for color grading.
#[derive(Debug, Clone)]
pub struct Lut3D {
    /// LUT size (e.g., 33 for 33x33x33).
    size:   u32,
    /// LUT data (flattened RGB values).
    data:   Vec<Color>,
    /// LUT name.
    name:   String,
    /// Interpolation mode.
    interp: LutInterpolation,
}

/// LUT interpolation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LutInterpolation {
    /// Nearest neighbor (fastest).
    Nearest,
    /// Trilinear interpolation (default).
    #[default]
    Trilinear,
    /// Tetrahedral interpolation (highest quality).
    Tetrahedral,
}

impl Lut3D {
    /// Creates a new identity LUT.
    #[must_use]
    pub fn identity(size: u32) -> Self {
        let total = (size * size * size) as usize;
        let mut data = Vec::with_capacity(total);

        for b in 0..size {
            for g in 0..size {
                for r in 0..size {
                    data.push(Color::rgb(
                        r as f32 / (size - 1) as f32,
                        g as f32 / (size - 1) as f32,
                        b as f32 / (size - 1) as f32,
                    ));
                }
            }
        }

        Self { size, data, name: "Identity".into(), interp: LutInterpolation::default() }
    }

    /// Returns the LUT size.
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// Returns the LUT name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the LUT name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Sets the interpolation mode.
    pub fn set_interpolation(&mut self, interp: LutInterpolation) {
        self.interp = interp;
    }

    /// Gets color at integer indices.
    fn get_at(&self, r: u32, g: u32, b: u32) -> &Color {
        let r = r.min(self.size - 1);
        let g = g.min(self.size - 1);
        let b = b.min(self.size - 1);
        let idx = (b * self.size * self.size + g * self.size + r) as usize;
        &self.data[idx.min(self.data.len() - 1)]
    }

    /// Applies the LUT to a color.
    #[must_use]
    pub fn apply(&self, color: &Color) -> Color {
        match self.interp {
            LutInterpolation::Nearest => self.apply_nearest(color),
            LutInterpolation::Trilinear => self.apply_trilinear(color),
            LutInterpolation::Tetrahedral => self.apply_trilinear(color), /* TODO: implement
                                                                           * tetrahedral */
        }
    }

    /// Applies using nearest neighbor.
    fn apply_nearest(&self, color: &Color) -> Color {
        let scale = (self.size - 1) as f32;
        let r = (color.r.clamp(0.0, 1.0) * scale + 0.5) as u32;
        let g = (color.g.clamp(0.0, 1.0) * scale + 0.5) as u32;
        let b = (color.b.clamp(0.0, 1.0) * scale + 0.5) as u32;

        let mut result = *self.get_at(r, g, b);
        result.a = color.a;
        result
    }

    /// Applies using trilinear interpolation.
    fn apply_trilinear(&self, color: &Color) -> Color {
        let scale = (self.size - 1) as f32;

        let r = (color.r.clamp(0.0, 1.0) * scale).min(scale);
        let g = (color.g.clamp(0.0, 1.0) * scale).min(scale);
        let b = (color.b.clamp(0.0, 1.0) * scale).min(scale);

        let r0 = r as u32;
        let g0 = g as u32;
        let b0 = b as u32;
        let r1 = (r0 + 1).min(self.size - 1);
        let g1 = (g0 + 1).min(self.size - 1);
        let b1 = (b0 + 1).min(self.size - 1);

        let fr = r - r0 as f32;
        let fg = g - g0 as f32;
        let fb = b - b0 as f32;

        // Trilinear interpolation
        let lerp = |a: f32, b: f32, t: f32| a + t * (b - a);
        let lerp_color = |a: &Color, b: &Color, t: f32| -> Color {
            Color::rgb(lerp(a.r, b.r, t), lerp(a.g, b.g, t), lerp(a.b, b.b, t))
        };

        let c000 = self.get_at(r0, g0, b0);
        let c100 = self.get_at(r1, g0, b0);
        let c010 = self.get_at(r0, g1, b0);
        let c110 = self.get_at(r1, g1, b0);
        let c001 = self.get_at(r0, g0, b1);
        let c101 = self.get_at(r1, g0, b1);
        let c011 = self.get_at(r0, g1, b1);
        let c111 = self.get_at(r1, g1, b1);

        let c00 = lerp_color(c000, c100, fr);
        let c10 = lerp_color(c010, c110, fr);
        let c01 = lerp_color(c001, c101, fr);
        let c11 = lerp_color(c011, c111, fr);

        let c0 = lerp_color(&c00, &c10, fg);
        let c1 = lerp_color(&c01, &c11, fg);

        let mut result = lerp_color(&c0, &c1, fb);
        result.a = color.a;
        result
    }
}

/// Complete color grading node.
#[derive(Debug, Clone)]
pub struct ColorGradingNode {
    /// Node name.
    pub name:          String,
    /// Whether node is enabled.
    pub enabled:       bool,
    /// Three-way color corrector.
    pub three_way:     ThreeWayCorrector,
    /// Color curves.
    pub curves:        ColorCurves,
    /// 3D LUT (optional).
    pub lut:           Option<Lut3D>,
    /// LUT intensity (0.0 to 1.0).
    pub lut_intensity: f32,
    /// Exposure adjustment (stops).
    pub exposure:      f32,
    /// Contrast (-1.0 to 1.0).
    pub contrast:      f32,
    /// Saturation (-1.0 to 1.0).
    pub saturation:    f32,
    /// Temperature adjustment (Kelvin offset).
    pub temperature:   f32,
    /// Tint adjustment (green-magenta).
    pub tint:          f32,
}

impl ColorGradingNode {
    /// Creates a new neutral grading node.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name:          name.into(),
            enabled:       true,
            three_way:     ThreeWayCorrector::new(),
            curves:        ColorCurves::new(),
            lut:           None,
            lut_intensity: 1.0,
            exposure:      0.0,
            contrast:      0.0,
            saturation:    0.0,
            temperature:   0.0,
            tint:          0.0,
        }
    }

    /// Applies the grading to a color.
    #[must_use]
    pub fn apply(&self, color: &Color) -> Color {
        if !self.enabled {
            return *color;
        }

        let mut result = *color;

        // Apply exposure (in stops)
        if self.exposure.abs() > f32::EPSILON {
            let mult = 2.0_f32.powf(self.exposure);
            result.r *= mult;
            result.g *= mult;
            result.b *= mult;
        }

        // Apply contrast
        if self.contrast.abs() > f32::EPSILON {
            let factor = (1.0 + self.contrast).max(0.0);
            result.r = (result.r - 0.5) * factor + 0.5;
            result.g = (result.g - 0.5) * factor + 0.5;
            result.b = (result.b - 0.5) * factor + 0.5;
        }

        // Apply three-way corrector
        result = self.three_way.apply(&result);

        // Apply curves
        result = self.curves.apply(&result);

        // Apply saturation
        if self.saturation.abs() > f32::EPSILON {
            let lum = result.luminance();
            let sat = 1.0 + self.saturation;
            result.r = lum + sat * (result.r - lum);
            result.g = lum + sat * (result.g - lum);
            result.b = lum + sat * (result.b - lum);
        }

        // Apply LUT
        if let Some(lut) = &self.lut {
            let lut_color = lut.apply(&result);
            // Blend with LUT intensity
            result.r = result.r + self.lut_intensity * (lut_color.r - result.r);
            result.g = result.g + self.lut_intensity * (lut_color.g - result.g);
            result.b = result.b + self.lut_intensity * (lut_color.b - result.b);
        }

        result
    }
}

impl Default for ColorGradingNode {
    fn default() -> Self {
        Self::new("Color Grading")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_luminance() {
        let white = Color::WHITE;
        assert!((white.luminance() - 1.0).abs() < 0.001);

        let black = Color::BLACK;
        assert!(black.luminance().abs() < 0.001);
    }

    #[test]
    fn test_color_hsl_roundtrip() {
        let original = Color::rgb(0.8, 0.3, 0.5);
        let hsl = original.to_hsl();
        let (h, s, l) = (hsl.h, hsl.s, hsl.l);
        let converted = Hsl::new(h, s, l).to_rgb();

        assert!((original.r - converted.r).abs() < 0.01);
        assert!((original.g - converted.g).abs() < 0.01);
        assert!((original.b - converted.b).abs() < 0.01);
    }

    #[test]
    fn test_color_curve() {
        let curve = ColorCurve::linear();
        assert!((curve.evaluate(0.5) - 0.5).abs() < 0.01);
        assert!((curve.evaluate(0.0) - 0.0).abs() < 0.01);
        assert!((curve.evaluate(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_lut_identity() {
        let lut = Lut3D::identity(17);
        let color = Color::rgb(0.5, 0.3, 0.7);
        let result = lut.apply(&color);

        assert!((result.r - color.r).abs() < 0.05);
        assert!((result.g - color.g).abs() < 0.05);
        assert!((result.b - color.b).abs() < 0.05);
    }

    #[test]
    fn test_grading_node_neutral() {
        let node = ColorGradingNode::default();
        let color = Color::rgb(0.5, 0.5, 0.5);
        let result = node.apply(&color);

        assert!((result.r - color.r).abs() < 0.01);
        assert!((result.g - color.g).abs() < 0.01);
        assert!((result.b - color.b).abs() < 0.01);
    }
}

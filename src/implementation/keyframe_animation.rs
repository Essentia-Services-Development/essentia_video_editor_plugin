//! Keyframe Animation System for Essentia Video Editor Plugin
//! GAP-220-B-006: Property Animation System
//!
//! Features: Keyframe management, interpolation, bezier curves,
//! expression support, and animated parameter control.

use crate::types::TimePosition;

/// Unique identifier for an animation track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationTrackId(u64);

impl AnimationTrackId {
    /// Creates a new animation track ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    #[must_use]
    pub const fn inner(&self) -> u64 {
        self.0
    }
}

/// Interpolation type between keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InterpolationType {
    /// No interpolation (step/hold).
    Hold,
    /// Linear interpolation.
    #[default]
    Linear,
    /// Bezier curve interpolation.
    Bezier,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    EaseOut,
    /// Ease in and out.
    EaseInOut,
    /// Cubic ease in.
    CubicIn,
    /// Cubic ease out.
    CubicOut,
    /// Cubic ease in-out.
    CubicInOut,
    /// Exponential ease in.
    ExponentialIn,
    /// Exponential ease out.
    ExponentialOut,
    /// Bounce effect.
    Bounce,
    /// Elastic effect.
    Elastic,
}

impl InterpolationType {
    /// Evaluates the easing function at t (0.0 to 1.0).
    #[must_use]
    pub fn evaluate(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Hold => 0.0,
            Self::Linear => t,
            Self::Bezier => t, // Bezier uses control points instead
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            },
            Self::CubicIn => t * t * t,
            Self::CubicOut => 1.0 - (1.0 - t).powi(3),
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            },
            Self::ExponentialIn => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f64.powf(10.0 * t - 10.0)
                }
            },
            Self::ExponentialOut => {
                if (t - 1.0).abs() < f64::EPSILON {
                    1.0
                } else {
                    1.0 - 2.0_f64.powf(-10.0 * t)
                }
            },
            Self::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let t = 1.0 - t;
                let bounce = if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t = t - 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    let t = t - 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d1;
                    n1 * t * t + 0.984375
                };
                1.0 - bounce
            },
            Self::Elastic => {
                let c4 = (2.0 * core::f64::consts::PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if (t - 1.0).abs() < f64::EPSILON {
                    1.0
                } else {
                    2.0_f64.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            },
        }
    }
}

/// Bezier curve handle for smooth keyframe interpolation.
#[derive(Debug, Clone, Copy, Default)]
pub struct BezierHandle {
    /// X offset from keyframe (time).
    pub x: f64,
    /// Y offset from keyframe (value).
    pub y: f64,
}

impl BezierHandle {
    /// Creates a new bezier handle.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Creates a flat/linear handle.
    #[must_use]
    pub const fn flat() -> Self {
        Self { x: 0.1, y: 0.0 }
    }
}

/// Value type for animated properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimatedValue {
    /// Single float value.
    Float(f64),
    /// 2D vector (position, scale).
    Vec2(f64, f64),
    /// 3D vector (position, rotation).
    Vec3(f64, f64, f64),
    /// 4D vector (color, quaternion).
    Vec4(f64, f64, f64, f64),
    /// Color (RGBA).
    Color(f32, f32, f32, f32),
    /// Boolean (for visibility, etc).
    Bool(bool),
    /// Integer.
    Int(i64),
}

impl AnimatedValue {
    /// Interpolates between two values.
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => Self::Float(a + t * (b - a)),
            (Self::Vec2(ax, ay), Self::Vec2(bx, by)) => {
                Self::Vec2(ax + t * (bx - ax), ay + t * (by - ay))
            },
            (Self::Vec3(ax, ay, az), Self::Vec3(bx, by, bz)) => {
                Self::Vec3(ax + t * (bx - ax), ay + t * (by - ay), az + t * (bz - az))
            },
            (Self::Vec4(ax, ay, az, aw), Self::Vec4(bx, by, bz, bw)) => Self::Vec4(
                ax + t * (bx - ax),
                ay + t * (by - ay),
                az + t * (bz - az),
                aw + t * (bw - aw),
            ),
            (Self::Color(ar, ag, ab, aa), Self::Color(br, bg, bb, ba)) => Self::Color(
                ar + t as f32 * (br - ar),
                ag + t as f32 * (bg - ag),
                ab + t as f32 * (bb - ab),
                aa + t as f32 * (ba - aa),
            ),
            (Self::Bool(a), Self::Bool(_)) => Self::Bool(*a), // Can't interpolate booleans
            (Self::Int(a), Self::Int(b)) => {
                Self::Int(((*a as f64) + t * (*b as f64 - *a as f64)) as i64)
            },
            _ => *self, // Mismatched types, return first
        }
    }

    /// Returns the value as f64 (for Float type).
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns the value as Vec2.
    #[must_use]
    pub fn as_vec2(&self) -> Option<(f64, f64)> {
        match self {
            Self::Vec2(x, y) => Some((*x, *y)),
            _ => None,
        }
    }
}

impl Default for AnimatedValue {
    fn default() -> Self {
        Self::Float(0.0)
    }
}

/// A single keyframe in an animation track.
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Time position of keyframe.
    time:          TimePosition,
    /// Value at this keyframe.
    value:         AnimatedValue,
    /// Interpolation to next keyframe.
    interpolation: InterpolationType,
    /// Incoming bezier handle.
    handle_in:     BezierHandle,
    /// Outgoing bezier handle.
    handle_out:    BezierHandle,
    /// Whether keyframe is selected (for UI).
    selected:      bool,
}

impl Keyframe {
    /// Creates a new keyframe.
    #[must_use]
    pub fn new(time: TimePosition, value: AnimatedValue) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationType::default(),
            handle_in: BezierHandle::flat(),
            handle_out: BezierHandle::flat(),
            selected: false,
        }
    }

    /// Returns the time position.
    #[must_use]
    pub const fn time(&self) -> TimePosition {
        self.time
    }

    /// Sets the time position.
    pub fn set_time(&mut self, time: TimePosition) {
        self.time = time;
    }

    /// Returns the value.
    #[must_use]
    pub const fn value(&self) -> &AnimatedValue {
        &self.value
    }

    /// Sets the value.
    pub fn set_value(&mut self, value: AnimatedValue) {
        self.value = value;
    }

    /// Returns the interpolation type.
    #[must_use]
    pub const fn interpolation(&self) -> InterpolationType {
        self.interpolation
    }

    /// Sets the interpolation type.
    pub fn set_interpolation(&mut self, interp: InterpolationType) {
        self.interpolation = interp;
    }

    /// Returns the incoming bezier handle.
    #[must_use]
    pub const fn handle_in(&self) -> &BezierHandle {
        &self.handle_in
    }

    /// Returns the outgoing bezier handle.
    #[must_use]
    pub const fn handle_out(&self) -> &BezierHandle {
        &self.handle_out
    }

    /// Sets bezier handles.
    pub fn set_handles(&mut self, handle_in: BezierHandle, handle_out: BezierHandle) {
        self.handle_in = handle_in;
        self.handle_out = handle_out;
    }

    /// Returns whether keyframe is selected.
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        self.selected
    }

    /// Sets selection state.
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

/// Animation track containing keyframes for a property.
#[derive(Debug, Clone)]
pub struct AnimationTrack {
    /// Track identifier.
    id:            AnimationTrackId,
    /// Property name being animated.
    property:      String,
    /// Keyframes (sorted by time).
    keyframes:     Vec<Keyframe>,
    /// Whether track is enabled.
    enabled:       bool,
    /// Whether track is muted (maintains values but doesn't animate).
    muted:         bool,
    /// Default value when no keyframes.
    default_value: AnimatedValue,
    /// Loop mode.
    loop_mode:     AnimationLoopMode,
}

/// Loop mode for animation tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnimationLoopMode {
    /// No looping (clamp to last value).
    #[default]
    None,
    /// Loop from end to start.
    Loop,
    /// Ping-pong (reverse at ends).
    PingPong,
    /// Continue with cycle offset.
    Cycle,
}

impl AnimationTrack {
    /// Creates a new animation track.
    #[must_use]
    pub fn new(
        id: AnimationTrackId, property: impl Into<String>, default_value: AnimatedValue,
    ) -> Self {
        Self {
            id,
            property: property.into(),
            keyframes: Vec::new(),
            enabled: true,
            muted: false,
            default_value,
            loop_mode: AnimationLoopMode::default(),
        }
    }

    /// Returns the track ID.
    #[must_use]
    pub const fn id(&self) -> AnimationTrackId {
        self.id
    }

    /// Returns the property name.
    #[must_use]
    pub fn property(&self) -> &str {
        &self.property
    }

    /// Returns whether the track is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables the track.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the track is muted.
    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    /// Sets muted state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Returns all keyframes.
    #[must_use]
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Returns mutable keyframes.
    pub fn keyframes_mut(&mut self) -> &mut Vec<Keyframe> {
        &mut self.keyframes
    }

    /// Returns the number of keyframes.
    #[must_use]
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Adds a keyframe at the specified time.
    pub fn add_keyframe(&mut self, time: TimePosition, value: AnimatedValue) -> usize {
        let keyframe = Keyframe::new(time, value);

        // Find insertion point (maintain sorted order)
        let pos = self
            .keyframes
            .iter()
            .position(|k| k.time().ms > time.ms)
            .unwrap_or(self.keyframes.len());

        // Check if keyframe already exists at this time
        if pos > 0 && self.keyframes[pos - 1].time().ms == time.ms {
            self.keyframes[pos - 1] = keyframe;
            pos - 1
        } else {
            self.keyframes.insert(pos, keyframe);
            pos
        }
    }

    /// Removes a keyframe at the specified index.
    pub fn remove_keyframe(&mut self, index: usize) -> Option<Keyframe> {
        if index < self.keyframes.len() {
            Some(self.keyframes.remove(index))
        } else {
            None
        }
    }

    /// Gets keyframe at index.
    #[must_use]
    pub fn get_keyframe(&self, index: usize) -> Option<&Keyframe> {
        self.keyframes.get(index)
    }

    /// Gets mutable keyframe at index.
    pub fn get_keyframe_mut(&mut self, index: usize) -> Option<&mut Keyframe> {
        self.keyframes.get_mut(index)
    }

    /// Finds keyframes around a time position.
    fn find_keyframes(&self, time: TimePosition) -> (Option<&Keyframe>, Option<&Keyframe>) {
        if self.keyframes.is_empty() {
            return (None, None);
        }

        // Find the first keyframe after or at this time
        let next_idx = self.keyframes.iter().position(|k| k.time().ms >= time.ms);

        match next_idx {
            None => (self.keyframes.last(), None), // After all keyframes
            Some(0) => (None, self.keyframes.first()), // Before all keyframes
            Some(i) => (Some(&self.keyframes[i - 1]), Some(&self.keyframes[i])),
        }
    }

    /// Evaluates the track at a time position.
    #[must_use]
    pub fn evaluate(&self, time: TimePosition) -> AnimatedValue {
        if !self.enabled || self.keyframes.is_empty() {
            return self.default_value;
        }

        if self.muted {
            // Return value at first keyframe when muted
            return self.keyframes[0].value;
        }

        let (prev, next) = self.find_keyframes(time);

        match (prev, next) {
            (None, None) => self.default_value,
            (None, Some(kf)) => kf.value, // Before first keyframe
            (Some(kf), None) => kf.value, // After last keyframe
            (Some(prev_kf), Some(next_kf)) => {
                if prev_kf.time().ms == time.ms {
                    return prev_kf.value;
                }

                // Handle hold interpolation
                if prev_kf.interpolation() == InterpolationType::Hold {
                    return prev_kf.value;
                }

                // Calculate interpolation factor
                let duration = (next_kf.time().ms - prev_kf.time().ms) as f64;
                let elapsed = (time.ms - prev_kf.time().ms) as f64;
                let t = if duration > 0.0 {
                    elapsed / duration
                } else {
                    0.0
                };

                // Apply easing
                let eased_t = if prev_kf.interpolation() == InterpolationType::Bezier {
                    self.evaluate_bezier(t, prev_kf, next_kf)
                } else {
                    prev_kf.interpolation().evaluate(t)
                };

                // Interpolate values
                prev_kf.value.lerp(&next_kf.value, eased_t)
            },
        }
    }

    /// Evaluates bezier interpolation.
    fn evaluate_bezier(&self, t: f64, prev: &Keyframe, next: &Keyframe) -> f64 {
        // Cubic bezier evaluation
        // P0 = (0, 0), P1 = prev.handle_out, P2 = (1-next.handle_in.x,
        // next.handle_in.y), P3 = (1, 1)
        let p1x = prev.handle_out().x.clamp(0.0, 1.0);
        let p1y = prev.handle_out().y;
        let p2x = (1.0 - next.handle_in().x).clamp(0.0, 1.0);
        let p2y = 1.0 - next.handle_in().y;

        // Newton-Raphson to find t for x
        let mut guess = t;
        for _ in 0..8 {
            let x = Self::bezier_component(guess, 0.0, p1x, p2x, 1.0);
            let dx = Self::bezier_derivative(guess, 0.0, p1x, p2x, 1.0);
            if dx.abs() < 1e-10 {
                break;
            }
            guess -= (x - t) / dx;
            guess = guess.clamp(0.0, 1.0);
        }

        // Get y value at the found t
        Self::bezier_component(guess, 0.0, p1y, p2y, 1.0)
    }

    /// Evaluates a cubic bezier component.
    fn bezier_component(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
        let mt = 1.0 - t;
        mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
    }

    /// Evaluates the derivative of a cubic bezier.
    fn bezier_derivative(t: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
        let mt = 1.0 - t;
        3.0 * mt * mt * (p1 - p0) + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
    }

    /// Returns the duration of the animation.
    #[must_use]
    pub fn duration(&self) -> TimePosition {
        self.keyframes.last().map(|k| k.time()).unwrap_or_default()
    }

    /// Clears all keyframes.
    pub fn clear(&mut self) {
        self.keyframes.clear();
    }

    /// Selects all keyframes in a time range.
    pub fn select_range(&mut self, start: TimePosition, end: TimePosition) {
        for kf in &mut self.keyframes {
            kf.selected = kf.time().ms >= start.ms && kf.time().ms <= end.ms;
        }
    }

    /// Clears all selections.
    pub fn clear_selection(&mut self) {
        for kf in &mut self.keyframes {
            kf.selected = false;
        }
    }

    /// Returns selected keyframe indices.
    #[must_use]
    pub fn selected_indices(&self) -> Vec<usize> {
        self.keyframes
            .iter()
            .enumerate()
            .filter(|(_, k)| k.selected)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Animation layer containing multiple tracks.
#[derive(Debug)]
pub struct AnimationLayer {
    /// Layer name.
    name:      String,
    /// Animation tracks.
    tracks:    Vec<AnimationTrack>,
    /// Next track ID.
    next_id:   u64,
    /// Target object ID.
    target_id: u64,
    /// Whether layer is enabled.
    enabled:   bool,
}

impl AnimationLayer {
    /// Creates a new animation layer.
    #[must_use]
    pub fn new(name: impl Into<String>, target_id: u64) -> Self {
        Self { name: name.into(), tracks: Vec::new(), next_id: 1, target_id, enabled: true }
    }

    /// Returns the layer name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the target object ID.
    #[must_use]
    pub const fn target_id(&self) -> u64 {
        self.target_id
    }

    /// Returns whether the layer is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Creates a new animation track for a property.
    pub fn create_track(
        &mut self, property: impl Into<String>, default_value: AnimatedValue,
    ) -> AnimationTrackId {
        let id = AnimationTrackId::new(self.next_id);
        self.next_id += 1;

        let track = AnimationTrack::new(id, property, default_value);
        self.tracks.push(track);
        id
    }

    /// Gets a track by property name.
    #[must_use]
    pub fn get_track_by_property(&self, property: &str) -> Option<&AnimationTrack> {
        self.tracks.iter().find(|t| t.property() == property)
    }

    /// Gets a mutable track by property name.
    pub fn get_track_by_property_mut(&mut self, property: &str) -> Option<&mut AnimationTrack> {
        self.tracks.iter_mut().find(|t| t.property() == property)
    }

    /// Gets a track by ID.
    #[must_use]
    pub fn get_track(&self, id: AnimationTrackId) -> Option<&AnimationTrack> {
        self.tracks.iter().find(|t| t.id() == id)
    }

    /// Gets a mutable track by ID.
    pub fn get_track_mut(&mut self, id: AnimationTrackId) -> Option<&mut AnimationTrack> {
        self.tracks.iter_mut().find(|t| t.id() == id)
    }

    /// Returns all tracks.
    #[must_use]
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Removes a track.
    pub fn remove_track(&mut self, id: AnimationTrackId) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id() == id) {
            self.tracks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Evaluates all tracks at a time position.
    #[must_use]
    pub fn evaluate_all(&self, time: TimePosition) -> Vec<(&str, AnimatedValue)> {
        if !self.enabled {
            return Vec::new();
        }

        self.tracks.iter().map(|t| (t.property(), t.evaluate(time))).collect()
    }

    /// Returns the total duration across all tracks.
    #[must_use]
    pub fn duration(&self) -> TimePosition {
        self.tracks
            .iter()
            .map(|t| t.duration())
            .max_by(|a, b| a.ms.cmp(&b.ms))
            .unwrap_or_default()
    }
}

/// Animation manager for the entire project.
pub struct AnimationManager {
    /// Animation layers.
    layers:   Vec<AnimationLayer>,
    /// Global animation settings.
    settings: AnimationSettings,
}

/// Global animation settings.
#[derive(Debug, Clone)]
pub struct AnimationSettings {
    /// Default interpolation type.
    pub default_interpolation: InterpolationType,
    /// Auto-keyframe mode.
    pub auto_keyframe:         bool,
    /// Keyframe snapping enabled.
    pub snap_keyframes:        bool,
    /// Snap threshold in frames.
    pub snap_threshold:        u32,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            default_interpolation: InterpolationType::Linear,
            auto_keyframe:         false,
            snap_keyframes:        true,
            snap_threshold:        2,
        }
    }
}

impl AnimationManager {
    /// Creates a new animation manager.
    #[must_use]
    pub fn new() -> Self {
        Self { layers: Vec::new(), settings: AnimationSettings::default() }
    }

    /// Creates a new animation layer.
    ///
    /// Creates a new animation layer and returns a mutable reference to it.
    ///
    /// # Errors
    ///
    /// Returns `VideoEditorError::Timeline` if the layer cannot be created.
    pub fn create_layer(
        &mut self, name: impl Into<String>, target_id: u64,
    ) -> crate::errors::VideoEditorResult<&mut AnimationLayer> {
        self.layers.push(AnimationLayer::new(name, target_id));
        // SAFETY: Element was just pushed, so last_mut will always succeed
        self.layers.last_mut().ok_or_else(|| {
            crate::VideoEditorError::Timeline("Layer was just added but not found".to_string())
        })
    }

    /// Gets an animation layer by target ID.
    #[must_use]
    pub fn get_layer(&self, target_id: u64) -> Option<&AnimationLayer> {
        self.layers.iter().find(|l| l.target_id() == target_id)
    }

    /// Gets a mutable animation layer by target ID.
    pub fn get_layer_mut(&mut self, target_id: u64) -> Option<&mut AnimationLayer> {
        self.layers.iter_mut().find(|l| l.target_id() == target_id)
    }

    /// Returns all layers.
    #[must_use]
    pub fn layers(&self) -> &[AnimationLayer] {
        &self.layers
    }

    /// Removes a layer by target ID.
    pub fn remove_layer(&mut self, target_id: u64) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.target_id() == target_id) {
            self.layers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns the animation settings.
    #[must_use]
    pub fn settings(&self) -> &AnimationSettings {
        &self.settings
    }

    /// Returns mutable animation settings.
    pub fn settings_mut(&mut self) -> &mut AnimationSettings {
        &mut self.settings
    }

    /// Evaluates all animations for a target at a time position.
    #[must_use]
    pub fn evaluate(&self, target_id: u64, time: TimePosition) -> Vec<(&str, AnimatedValue)> {
        self.get_layer(target_id).map(|l| l.evaluate_all(time)).unwrap_or_default()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_types() {
        let linear = InterpolationType::Linear;
        assert!((linear.evaluate(0.5) - 0.5).abs() < 0.001);

        let ease_in = InterpolationType::EaseIn;
        assert!(ease_in.evaluate(0.5) < 0.5); // Slow start
    }

    #[test]
    fn test_animated_value_lerp() {
        let a = AnimatedValue::Float(0.0);
        let b = AnimatedValue::Float(10.0);
        let result = a.lerp(&b, 0.5);

        assert!(matches!(result, AnimatedValue::Float(v) if (v - 5.0).abs() < 0.001));
    }

    #[test]
    fn test_animation_track() {
        let mut track = AnimationTrack::new(
            AnimationTrackId::new(1),
            "position.x",
            AnimatedValue::Float(0.0),
        );

        track.add_keyframe(TimePosition::from_ms(0), AnimatedValue::Float(0.0));
        track.add_keyframe(TimePosition::from_ms(1000), AnimatedValue::Float(100.0));

        let value = track.evaluate(TimePosition::from_ms(500));
        assert!(matches!(value, AnimatedValue::Float(v) if (v - 50.0).abs() < 1.0));
    }

    #[test]
    fn test_keyframe_ordering() {
        let mut track = AnimationTrack::new(
            AnimationTrackId::new(1),
            "opacity",
            AnimatedValue::Float(1.0),
        );

        // Add keyframes out of order
        track.add_keyframe(TimePosition::from_ms(1000), AnimatedValue::Float(0.5));
        track.add_keyframe(TimePosition::from_ms(0), AnimatedValue::Float(0.0));
        track.add_keyframe(TimePosition::from_ms(500), AnimatedValue::Float(0.25));

        // Should be sorted
        assert_eq!(track.keyframes()[0].time().ms, 0);
        assert_eq!(track.keyframes()[1].time().ms, 500);
        assert_eq!(track.keyframes()[2].time().ms, 1000);
    }

    #[test]
    fn test_animation_layer() {
        let mut layer = AnimationLayer::new("Transform", 1);
        let track_id = layer.create_track("position.x", AnimatedValue::Float(0.0));

        assert!(layer.get_track(track_id).is_some());
        assert!(layer.get_track_by_property("position.x").is_some());
    }
}

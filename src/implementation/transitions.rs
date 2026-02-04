//! Video Transitions for Essentia Video Editor Plugin
//! GAP-220-B-001: Transition Effects
//!
//! Features: CrossFade, Wipe, Dissolve, Push, Slide, Zoom transitions
//! with configurable duration, easing, and parameters.

use crate::{
    errors::{VideoEditorError, VideoEditorResult},
    types::TimePosition,
};

/// Unique identifier for a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionId(u64);

impl TransitionId {
    /// Creates a new transition ID.
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

/// Transition type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransitionType {
    /// Cross-fade between clips.
    #[default]
    CrossFade,
    /// Cross-dissolve with luminance blending.
    CrossDissolve,
    /// Fade to black then to next clip.
    FadeToBlack,
    /// Fade to white then to next clip.
    FadeToWhite,
    /// Wipe transition in specified direction.
    Wipe(WipeDirection),
    /// Push transition.
    Push(PushDirection),
    /// Slide transition.
    Slide(SlideDirection),
    /// Zoom transition.
    Zoom(ZoomType),
    /// Iris transition.
    Iris(IrisShape),
    /// Clock wipe.
    ClockWipe(ClockDirection),
    /// Page turn effect.
    PageTurn(PageTurnDirection),
    /// Cube rotation.
    CubeRotate(CubeAxis),
    /// Custom shader transition.
    CustomShader(u64),
}

/// Wipe direction for wipe transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WipeDirection {
    /// Left to right.
    #[default]
    LeftToRight,
    /// Right to left.
    RightToLeft,
    /// Top to bottom.
    TopToBottom,
    /// Bottom to top.
    BottomToTop,
    /// Diagonal top-left to bottom-right.
    DiagonalTLBR,
    /// Diagonal top-right to bottom-left.
    DiagonalTRBL,
    /// Center outward.
    CenterOut,
    /// Outside inward.
    OutsideIn,
}

/// Push direction for push transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PushDirection {
    /// Push from left.
    #[default]
    Left,
    /// Push from right.
    Right,
    /// Push from top.
    Top,
    /// Push from bottom.
    Bottom,
}

/// Slide direction for slide transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SlideDirection {
    /// Slide from left.
    #[default]
    Left,
    /// Slide from right.
    Right,
    /// Slide from top.
    Top,
    /// Slide from bottom.
    Bottom,
}

/// Zoom type for zoom transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoomType {
    /// Zoom in to center.
    #[default]
    ZoomIn,
    /// Zoom out from center.
    ZoomOut,
    /// Cross zoom (zoom both clips).
    CrossZoom,
}

/// Iris shape for iris transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IrisShape {
    /// Circular iris.
    #[default]
    Circle,
    /// Rectangular iris.
    Rectangle,
    /// Diamond iris.
    Diamond,
    /// Star iris.
    Star,
    /// Heart iris.
    Heart,
}

/// Clock wipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ClockDirection {
    /// Clockwise from 12 o'clock.
    #[default]
    Clockwise,
    /// Counter-clockwise from 12 o'clock.
    CounterClockwise,
}

/// Page turn direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageTurnDirection {
    /// Turn from right (like reading a book).
    #[default]
    Right,
    /// Turn from left.
    Left,
    /// Turn from top.
    Top,
    /// Turn from bottom.
    Bottom,
}

/// Cube rotation axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CubeAxis {
    /// Rotate around horizontal axis (flip vertically).
    #[default]
    Horizontal,
    /// Rotate around vertical axis (flip horizontally).
    Vertical,
}

/// Easing function for transition timing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransitionEasing {
    /// Linear interpolation.
    #[default]
    Linear,
    /// Ease in (slow start).
    EaseIn,
    /// Ease out (slow end).
    EaseOut,
    /// Ease in and out.
    EaseInOut,
    /// Quadratic ease in.
    QuadIn,
    /// Quadratic ease out.
    QuadOut,
    /// Quadratic ease in-out.
    QuadInOut,
    /// Cubic ease in.
    CubicIn,
    /// Cubic ease out.
    CubicOut,
    /// Cubic ease in-out.
    CubicInOut,
    /// Exponential ease in.
    ExpoIn,
    /// Exponential ease out.
    ExpoOut,
    /// Elastic bounce.
    Elastic,
    /// Bounce effect.
    Bounce,
}

impl TransitionEasing {
    /// Applies the easing function to a progress value (0.0 to 1.0).
    #[must_use]
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            },
            Self::QuadIn => t * t,
            Self::QuadOut => 1.0 - (1.0 - t).powi(2),
            Self::QuadInOut => {
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
            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0_f64.powf(10.0 * t - 10.0)
                }
            },
            Self::ExpoOut => {
                if (t - 1.0).abs() < f64::EPSILON {
                    1.0
                } else {
                    1.0 - 2.0_f64.powf(-10.0 * t)
                }
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
            Self::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                let mut t = t;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    t -= 1.5 / d1;
                    n1 * t * t + 0.75
                } else if t < 2.5 / d1 {
                    t -= 2.25 / d1;
                    n1 * t * t + 0.9375
                } else {
                    t -= 2.625 / d1;
                    n1 * t * t + 0.984375
                }
            },
        }
    }
}

/// A video transition between two clips.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Unique transition identifier.
    id:              TransitionId,
    /// Transition type.
    transition_type: TransitionType,
    /// Transition duration.
    duration:        TimePosition,
    /// Easing function.
    easing:          TransitionEasing,
    /// Transition progress (0.0 to 1.0).
    progress:        f64,
    /// Custom parameters.
    parameters:      TransitionParameters,
    /// Whether transition is enabled.
    enabled:         bool,
}

/// Custom parameters for transition configuration.
#[derive(Debug, Clone, Default)]
pub struct TransitionParameters {
    /// Softness of edge (0.0 = hard, 1.0 = soft).
    pub edge_softness:   f64,
    /// Blur amount during transition.
    pub blur_amount:     f64,
    /// Feather size in pixels.
    pub feather:         f64,
    /// Border width (for some transitions).
    pub border_width:    f64,
    /// Border color (RGBA).
    pub border_color:    [f32; 4],
    /// Center point for iris/zoom (normalized).
    pub center:          [f64; 2],
    /// Rotation angle in degrees.
    pub rotation:        f64,
    /// Custom shader ID (if applicable).
    pub custom_shader:   Option<u64>,
    /// Audio crossfade enabled.
    pub audio_crossfade: bool,
}

impl Transition {
    /// Creates a new transition.
    #[must_use]
    pub fn new(id: TransitionId, transition_type: TransitionType, duration: TimePosition) -> Self {
        Self {
            id,
            transition_type,
            duration,
            easing: TransitionEasing::default(),
            progress: 0.0,
            parameters: TransitionParameters::default(),
            enabled: true,
        }
    }

    /// Creates a cross-fade transition with default settings.
    #[must_use]
    pub fn crossfade(id: TransitionId, duration: TimePosition) -> Self {
        Self::new(id, TransitionType::CrossFade, duration)
    }

    /// Creates a wipe transition.
    #[must_use]
    pub fn wipe(id: TransitionId, duration: TimePosition, direction: WipeDirection) -> Self {
        Self::new(id, TransitionType::Wipe(direction), duration)
    }

    /// Returns the transition ID.
    #[must_use]
    pub const fn id(&self) -> TransitionId {
        self.id
    }

    /// Returns the transition type.
    #[must_use]
    pub const fn transition_type(&self) -> TransitionType {
        self.transition_type
    }

    /// Returns the transition duration.
    #[must_use]
    pub const fn duration(&self) -> TimePosition {
        self.duration
    }

    /// Sets the transition duration.
    pub fn set_duration(&mut self, duration: TimePosition) {
        self.duration = duration;
    }

    /// Returns the easing function.
    #[must_use]
    pub const fn easing(&self) -> TransitionEasing {
        self.easing
    }

    /// Sets the easing function.
    pub fn set_easing(&mut self, easing: TransitionEasing) {
        self.easing = easing;
    }

    /// Returns current progress (0.0 to 1.0).
    #[must_use]
    pub const fn progress(&self) -> f64 {
        self.progress
    }

    /// Sets the progress value.
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Returns the eased progress value.
    #[must_use]
    pub fn eased_progress(&self) -> f64 {
        self.easing.apply(self.progress)
    }

    /// Returns the parameters.
    #[must_use]
    pub const fn parameters(&self) -> &TransitionParameters {
        &self.parameters
    }

    /// Returns mutable parameters.
    pub fn parameters_mut(&mut self) -> &mut TransitionParameters {
        &mut self.parameters
    }

    /// Returns whether the transition is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables the transition.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Updates progress based on current time position.
    pub fn update(&mut self, current_time: TimePosition, start_time: TimePosition) {
        if self.duration.ms == 0 {
            self.progress = 1.0;
            return;
        }

        let elapsed = current_time.ms.saturating_sub(start_time.ms);
        self.progress = (elapsed as f64 / self.duration.ms as f64).clamp(0.0, 1.0);
    }

    /// Returns whether the transition is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        (self.progress - 1.0).abs() < f64::EPSILON
    }

    /// Resets the transition to the beginning.
    pub fn reset(&mut self) {
        self.progress = 0.0;
    }
}

/// Manager for video transitions.
pub struct TransitionManager {
    /// All transitions in the project.
    transitions:      Vec<TransitionPlacement>,
    /// Next transition ID.
    next_id:          u64,
    /// Default transition type.
    default_type:     TransitionType,
    /// Default transition duration.
    default_duration: TimePosition,
    /// Preset transitions.
    presets:          Vec<TransitionPreset>,
}

/// A transition placed between two clips.
#[derive(Debug, Clone)]
pub struct TransitionPlacement {
    /// The transition.
    pub transition: Transition,
    /// Track ID where transition is placed.
    pub track_id:   u64,
    /// Clip ID of the outgoing clip.
    pub clip_a_id:  u64,
    /// Clip ID of the incoming clip.
    pub clip_b_id:  u64,
    /// Start time of transition.
    pub start_time: TimePosition,
}

/// A saved transition preset.
#[derive(Debug, Clone)]
pub struct TransitionPreset {
    /// Preset name.
    pub name:            String,
    /// Transition type.
    pub transition_type: TransitionType,
    /// Duration.
    pub duration:        TimePosition,
    /// Easing.
    pub easing:          TransitionEasing,
    /// Parameters.
    pub parameters:      TransitionParameters,
}

impl TransitionManager {
    /// Creates a new transition manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transitions:      Vec::new(),
            next_id:          1,
            default_type:     TransitionType::CrossFade,
            default_duration: TimePosition::from_ms(500),
            presets:          Self::create_default_presets(),
        }
    }

    /// Creates default transition presets.
    fn create_default_presets() -> Vec<TransitionPreset> {
        vec![
            TransitionPreset {
                name:            "Quick Fade".into(),
                transition_type: TransitionType::CrossFade,
                duration:        TimePosition::from_ms(250),
                easing:          TransitionEasing::Linear,
                parameters:      TransitionParameters::default(),
            },
            TransitionPreset {
                name:            "Smooth Dissolve".into(),
                transition_type: TransitionType::CrossDissolve,
                duration:        TimePosition::from_ms(1000),
                easing:          TransitionEasing::EaseInOut,
                parameters:      TransitionParameters::default(),
            },
            TransitionPreset {
                name:            "Wipe Left".into(),
                transition_type: TransitionType::Wipe(WipeDirection::LeftToRight),
                duration:        TimePosition::from_ms(500),
                easing:          TransitionEasing::EaseOut,
                parameters:      TransitionParameters { edge_softness: 0.1, ..Default::default() },
            },
            TransitionPreset {
                name:            "Push Right".into(),
                transition_type: TransitionType::Push(PushDirection::Right),
                duration:        TimePosition::from_ms(500),
                easing:          TransitionEasing::CubicOut,
                parameters:      TransitionParameters::default(),
            },
            TransitionPreset {
                name:            "Zoom Blur".into(),
                transition_type: TransitionType::Zoom(ZoomType::CrossZoom),
                duration:        TimePosition::from_ms(750),
                easing:          TransitionEasing::QuadInOut,
                parameters:      TransitionParameters { blur_amount: 0.3, ..Default::default() },
            },
        ]
    }

    /// Generates a new transition ID.
    fn next_id(&mut self) -> TransitionId {
        let id = TransitionId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds a transition between two clips.
    pub fn add_transition(
        &mut self, track_id: u64, clip_a_id: u64, clip_b_id: u64, start_time: TimePosition,
        transition_type: Option<TransitionType>, duration: Option<TimePosition>,
    ) -> TransitionId {
        let id = self.next_id();
        let trans_type = transition_type.unwrap_or(self.default_type);
        let dur = duration.unwrap_or(self.default_duration);

        let transition = Transition::new(id, trans_type, dur);
        let placement =
            TransitionPlacement { transition, track_id, clip_a_id, clip_b_id, start_time };

        self.transitions.push(placement);
        id
    }

    /// Adds a transition from a preset.
    pub fn add_from_preset(
        &mut self, track_id: u64, clip_a_id: u64, clip_b_id: u64, start_time: TimePosition,
        preset_name: &str,
    ) -> VideoEditorResult<TransitionId> {
        let preset = self
            .presets
            .iter()
            .find(|p| p.name == preset_name)
            .ok_or_else(|| VideoEditorError::Effect(format!("Preset not found: {preset_name}")))?
            .clone();

        let id = self.next_id();
        let mut transition = Transition::new(id, preset.transition_type, preset.duration);
        transition.set_easing(preset.easing);
        *transition.parameters_mut() = preset.parameters;

        let placement =
            TransitionPlacement { transition, track_id, clip_a_id, clip_b_id, start_time };

        self.transitions.push(placement);
        Ok(id)
    }

    /// Removes a transition.
    pub fn remove_transition(&mut self, id: TransitionId) -> bool {
        if let Some(pos) = self.transitions.iter().position(|t| t.transition.id() == id) {
            self.transitions.remove(pos);
            true
        } else {
            false
        }
    }

    /// Gets a transition by ID.
    #[must_use]
    pub fn get_transition(&self, id: TransitionId) -> Option<&TransitionPlacement> {
        self.transitions.iter().find(|t| t.transition.id() == id)
    }

    /// Gets a mutable transition by ID.
    pub fn get_transition_mut(&mut self, id: TransitionId) -> Option<&mut TransitionPlacement> {
        self.transitions.iter_mut().find(|t| t.transition.id() == id)
    }

    /// Gets all transitions for a track.
    #[must_use]
    pub fn transitions_for_track(&self, track_id: u64) -> Vec<&TransitionPlacement> {
        self.transitions.iter().filter(|t| t.track_id == track_id).collect()
    }

    /// Gets transition at a specific time on a track.
    #[must_use]
    pub fn transition_at_time(
        &self, track_id: u64, time: TimePosition,
    ) -> Option<&TransitionPlacement> {
        self.transitions.iter().find(|t| {
            t.track_id == track_id
                && time.ms >= t.start_time.ms
                && time.ms < t.start_time.ms + t.transition.duration().ms
        })
    }

    /// Updates all transitions for the current time.
    pub fn update_all(&mut self, current_time: TimePosition) {
        for placement in &mut self.transitions {
            placement.transition.update(current_time, placement.start_time);
        }
    }

    /// Returns all transition placements.
    #[must_use]
    pub fn all_transitions(&self) -> &[TransitionPlacement] {
        &self.transitions
    }

    /// Returns available presets.
    #[must_use]
    pub fn presets(&self) -> &[TransitionPreset] {
        &self.presets
    }

    /// Adds a custom preset.
    pub fn add_preset(&mut self, preset: TransitionPreset) {
        self.presets.push(preset);
    }

    /// Sets the default transition type.
    pub fn set_default_type(&mut self, transition_type: TransitionType) {
        self.default_type = transition_type;
    }

    /// Sets the default transition duration.
    pub fn set_default_duration(&mut self, duration: TimePosition) {
        self.default_duration = duration;
    }
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_creation() {
        let id = TransitionId::new(1);
        let duration = TimePosition::from_ms(500);
        let transition = Transition::crossfade(id, duration);

        assert_eq!(transition.id().inner(), 1);
        assert!(matches!(
            transition.transition_type(),
            TransitionType::CrossFade
        ));
        assert_eq!(transition.duration().ms, 500);
    }

    #[test]
    fn test_easing_functions() {
        let linear = TransitionEasing::Linear;
        assert!((linear.apply(0.5) - 0.5).abs() < 0.001);

        let ease_in = TransitionEasing::EaseIn;
        assert!(ease_in.apply(0.5) < 0.5); // Slow start means less than linear at midpoint
    }

    #[test]
    fn test_transition_manager() {
        let mut manager = TransitionManager::new();
        let id = manager.add_transition(1, 1, 2, TimePosition::from_ms(1000), None, None);

        assert!(manager.get_transition(id).is_some());
        assert!(manager.remove_transition(id));
        assert!(manager.get_transition(id).is_none());
    }

    #[test]
    fn test_transition_progress() {
        let mut transition =
            Transition::crossfade(TransitionId::new(1), TimePosition::from_ms(1000));

        transition.update(TimePosition::from_ms(500), TimePosition::from_ms(0));
        assert!((transition.progress() - 0.5).abs() < 0.001);

        transition.update(TimePosition::from_ms(1000), TimePosition::from_ms(0));
        assert!(transition.is_complete());
    }
}

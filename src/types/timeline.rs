//! Timeline types for NLE editing.
//!
//! Multi-track timeline, clip references, and track types.

use super::core::TimePosition;

/// Track type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrackType {
    /// Video track.
    #[default]
    Video,
    /// Audio track.
    Audio,
    /// Subtitle/caption track.
    Subtitle,
    /// Data/metadata track.
    Data,
    /// Effects/adjustment track.
    Effect,
}

impl TrackType {
    /// Returns the track type name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Subtitle => "Subtitle",
            Self::Data => "Data",
            Self::Effect => "Effect",
        }
    }

    /// Returns whether this track can hold video clips.
    #[must_use]
    pub const fn accepts_video(&self) -> bool {
        matches!(self, Self::Video | Self::Effect)
    }

    /// Returns whether this track can hold audio clips.
    #[must_use]
    pub const fn accepts_audio(&self) -> bool {
        matches!(self, Self::Audio)
    }
}

/// Timeline track representing a single horizontal lane.
#[derive(Debug, Clone)]
pub struct TimelineTrack {
    /// Unique track identifier.
    pub id:         u64,
    /// Track name.
    pub name:       String,
    /// Track type.
    pub track_type: TrackType,
    /// Track index (vertical position).
    pub index:      usize,
    /// Whether track is enabled.
    pub enabled:    bool,
    /// Whether track is locked (prevents editing).
    pub locked:     bool,
    /// Whether track is muted (for audio).
    pub muted:      bool,
    /// Whether track is soloed (only this plays).
    pub solo:       bool,
    /// Track height in pixels (for UI).
    pub height:     u32,
    /// Track clips.
    pub clips:      Vec<TimelineClip>,
}

impl TimelineTrack {
    /// Creates a new timeline track.
    #[must_use]
    pub fn new(id: u64, name: impl Into<String>, track_type: TrackType, index: usize) -> Self {
        Self {
            id,
            name: name.into(),
            track_type,
            index,
            enabled: true,
            locked: false,
            muted: false,
            solo: false,
            height: 64,
            clips: Vec::new(),
        }
    }

    /// Adds a clip to the track.
    pub fn add_clip(&mut self, clip: TimelineClip) {
        self.clips.push(clip);
        self.clips.sort_by(|a, b| a.start.ms.cmp(&b.start.ms));
    }

    /// Removes a clip by ID.
    pub fn remove_clip(&mut self, clip_id: u64) -> Option<TimelineClip> {
        if let Some(pos) = self.clips.iter().position(|c| c.id == clip_id) {
            Some(self.clips.remove(pos))
        } else {
            None
        }
    }

    /// Returns the total duration of all clips.
    #[must_use]
    pub fn duration(&self) -> TimePosition {
        self.clips.last().map(|c| c.end()).unwrap_or_default()
    }

    /// Checks if a time range is available (no overlapping clips).
    #[must_use]
    pub fn is_range_available(&self, start: TimePosition, end: TimePosition) -> bool {
        !self.clips.iter().any(|c| {
            let clip_end = c.end();
            start.ms < clip_end.ms && end.ms > c.start.ms
        })
    }
}

/// Timeline clip reference.
#[derive(Debug, Clone)]
pub struct TimelineClip {
    /// Unique clip identifier.
    pub id:        u64,
    /// Start position on timeline.
    pub start:     TimePosition,
    /// Clip duration.
    pub duration:  TimePosition,
    /// Source media ID.
    pub source_id: u64,
    /// In point (trim start).
    pub in_point:  TimePosition,
    /// Out point (trim end).
    pub out_point: TimePosition,
    /// Playback speed multiplier.
    pub speed:     f32,
    /// Whether clip is enabled.
    pub enabled:   bool,
    /// Clip name.
    pub name:      String,
}

impl TimelineClip {
    /// Creates a new timeline clip.
    #[must_use]
    pub fn new(id: u64, source_id: u64, start: TimePosition, duration: TimePosition) -> Self {
        Self {
            id,
            start,
            duration,
            source_id,
            in_point: TimePosition::default(),
            out_point: duration,
            speed: 1.0,
            enabled: true,
            name: String::new(),
        }
    }

    /// Returns the end position of the clip.
    #[must_use]
    pub fn end(&self) -> TimePosition {
        TimePosition::from_ms(self.start.ms + self.duration.ms)
    }

    /// Returns the effective duration considering speed.
    #[must_use]
    pub fn effective_duration(&self) -> TimePosition {
        let source_dur = self.out_point.ms.saturating_sub(self.in_point.ms);
        let effective = (source_dur as f64 / self.speed as f64) as u64;
        TimePosition::from_ms(effective)
    }

    /// Splits the clip at the given position.
    #[must_use]
    pub fn split_at(&self, position: TimePosition, new_id: u64) -> Option<(Self, Self)> {
        if position.ms <= self.start.ms || position.ms >= self.end().ms {
            return None;
        }

        let split_offset = position.ms - self.start.ms;
        let source_split = self.in_point.ms + (split_offset as f64 * self.speed as f64) as u64;

        let first = Self {
            id:        self.id,
            start:     self.start,
            duration:  TimePosition::from_ms(split_offset),
            source_id: self.source_id,
            in_point:  self.in_point,
            out_point: TimePosition::from_ms(source_split),
            speed:     self.speed,
            enabled:   self.enabled,
            name:      self.name.clone(),
        };

        let second = Self {
            id:        new_id,
            start:     position,
            duration:  TimePosition::from_ms(self.duration.ms - split_offset),
            source_id: self.source_id,
            in_point:  TimePosition::from_ms(source_split),
            out_point: self.out_point,
            speed:     self.speed,
            enabled:   self.enabled,
            name:      self.name.clone(),
        };

        Some((first, second))
    }

    /// Checks if the clip contains the given position.
    #[must_use]
    pub fn contains(&self, position: TimePosition) -> bool {
        position.ms >= self.start.ms && position.ms < self.end().ms
    }
}

/// Timeline position alias for compatibility.
pub type TimelinePosition = TimePosition;

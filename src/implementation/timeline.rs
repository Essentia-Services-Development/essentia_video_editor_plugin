//! Timeline management.

use crate::types::{TimelinePosition, TimelineTrack, TrackType};

/// Timeline manager.
pub struct TimelineManager {
    tracks:        Vec<TimelineTrack>,
    next_track_id: u64,
    duration:      TimelinePosition,
}

impl TimelineManager {
    /// Create a new timeline manager.
    pub fn new() -> Self {
        Self {
            tracks:        Vec::new(),
            next_track_id: 1,
            duration:      TimelinePosition::default(),
        }
    }

    /// Add a new track.
    pub fn add_track(&mut self, name: impl Into<String>, track_type: TrackType) -> u64 {
        let id = self.next_track_id;
        self.next_track_id += 1;
        let index = self.tracks.len();

        self.tracks.push(TimelineTrack::new(id, name, track_type, index));

        id
    }

    /// Remove a track.
    pub fn remove_track(&mut self, track_id: u64) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == track_id) {
            self.tracks.remove(pos);
            // Reindex remaining tracks
            for (i, track) in self.tracks.iter_mut().enumerate() {
                track.index = i;
            }
            self.recalculate_duration();
            true
        } else {
            false
        }
    }

    /// Get all tracks.
    pub fn tracks(&self) -> &[TimelineTrack] {
        &self.tracks
    }

    /// Get mutable access to tracks.
    pub fn tracks_mut(&mut self) -> &mut Vec<TimelineTrack> {
        &mut self.tracks
    }

    /// Get timeline duration.
    pub fn duration(&self) -> TimelinePosition {
        self.duration
    }

    /// Get timeline duration in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.duration.ms
    }

    /// Update timeline duration based on clips.
    pub fn recalculate_duration(&mut self) {
        self.duration = self
            .tracks
            .iter()
            .map(|t| t.duration())
            .max_by(|a, b| a.ms.cmp(&b.ms))
            .unwrap_or_default();
    }

    /// Get a track by ID.
    pub fn get_track(&self, track_id: u64) -> Option<&TimelineTrack> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    /// Get a mutable track by ID.
    pub fn get_track_mut(&mut self, track_id: u64) -> Option<&mut TimelineTrack> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }
}

impl Default for TimelineManager {
    fn default() -> Self {
        Self::new()
    }
}

//! Timeline management.

use crate::errors::VideoEditorResult;
use crate::types::{TimelineTrack, TrackType};

/// Timeline manager.
pub struct TimelineManager {
    tracks: Vec<TimelineTrack>,
    next_track_id: u64,
    duration_ms: u64,
}

impl TimelineManager {
    /// Create a new timeline manager.
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            next_track_id: 1,
            duration_ms: 0,
        }
    }

    /// Add a new track.
    pub fn add_track(&mut self, name: impl Into<String>, track_type: TrackType) -> u64 {
        let id = self.next_track_id;
        self.next_track_id += 1;

        self.tracks.push(TimelineTrack {
            id,
            name: name.into(),
            track_type,
            muted: false,
            locked: false,
            clips: Vec::new(),
        });

        id
    }

    /// Remove a track.
    pub fn remove_track(&mut self, track_id: u64) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == track_id) {
            self.tracks.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all tracks.
    pub fn tracks(&self) -> &[TimelineTrack] {
        &self.tracks
    }

    /// Get timeline duration.
    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }

    /// Update timeline duration based on clips.
    pub fn recalculate_duration(&mut self) {
        self.duration_ms = self.tracks
            .iter()
            .flat_map(|t| &t.clips)
            .map(|c| c.position.ms + c.duration_ms)
            .max()
            .unwrap_or(0);
    }
}

impl Default for TimelineManager {
    fn default() -> Self {
        Self::new()
    }
}

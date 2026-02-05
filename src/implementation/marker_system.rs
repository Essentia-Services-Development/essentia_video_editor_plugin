//! Marker System for Essentia Video Editor Plugin
//! GAP-220-B-007: Timeline Markers and Chapters
//!
//! Features: Marker types, marker filtering, chapters,
//! import/export, and navigation helpers.

use crate::{
    errors::{VideoEditorError, VideoEditorResult},
    types::{TimePosition, Timestamp},
};

/// Unique identifier for a marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarkerId(u64);

impl MarkerId {
    /// Creates a new marker ID.
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

/// Type of marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MarkerType {
    /// General marker.
    #[default]
    Standard,
    /// Chapter marker (for export).
    Chapter,
    /// Sync point (for audio sync).
    SyncPoint,
    /// Comment/note marker.
    Comment,
    /// To-do item.
    Todo,
    /// Approved marker.
    Approved,
    /// Needs review marker.
    NeedsReview,
    /// In point marker.
    InPoint,
    /// Out point marker.
    OutPoint,
    /// Beat marker (for music sync).
    Beat,
    /// Cue point (for live events).
    Cue,
}

impl MarkerType {
    /// Returns the display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Standard => "Marker",
            Self::Chapter => "Chapter",
            Self::SyncPoint => "Sync Point",
            Self::Comment => "Comment",
            Self::Todo => "To-Do",
            Self::Approved => "Approved",
            Self::NeedsReview => "Needs Review",
            Self::InPoint => "In Point",
            Self::OutPoint => "Out Point",
            Self::Beat => "Beat",
            Self::Cue => "Cue",
        }
    }

    /// Returns the default color (RGBA).
    #[must_use]
    pub const fn default_color(&self) -> [f32; 4] {
        match self {
            Self::Standard => [0.2, 0.6, 1.0, 1.0],    // Blue
            Self::Chapter => [0.2, 0.8, 0.2, 1.0],     // Green
            Self::SyncPoint => [1.0, 0.5, 0.0, 1.0],   // Orange
            Self::Comment => [1.0, 1.0, 0.2, 1.0],     // Yellow
            Self::Todo => [1.0, 0.2, 0.2, 1.0],        // Red
            Self::Approved => [0.2, 0.9, 0.5, 1.0],    // Bright green
            Self::NeedsReview => [0.9, 0.6, 0.1, 1.0], // Amber
            Self::InPoint => [0.0, 1.0, 0.5, 1.0],     // Cyan-ish
            Self::OutPoint => [1.0, 0.0, 0.5, 1.0],    // Magenta-ish
            Self::Beat => [0.8, 0.2, 0.8, 1.0],        // Purple
            Self::Cue => [0.5, 0.5, 1.0, 1.0],         // Light blue
        }
    }
}

/// A marker on the timeline.
#[derive(Debug, Clone)]
pub struct Marker {
    /// Unique identifier.
    id:          MarkerId,
    /// Time position.
    position:    TimePosition,
    /// Duration (for range markers, 0 for point markers).
    duration:    TimePosition,
    /// Marker type.
    marker_type: MarkerType,
    /// Marker name/label.
    name:        String,
    /// Comment/description.
    comment:     String,
    /// Color override (RGBA).
    color:       Option<[f32; 4]>,
    /// Tags for filtering.
    tags:        Vec<String>,
    /// Whether marker is locked.
    locked:      bool,
    /// Creation timestamp.
    created_at:  Timestamp,
    /// Last modified timestamp.
    modified_at: Timestamp,
    /// Author/creator.
    author:      Option<String>,
}

impl Marker {
    /// Creates a new marker.
    #[must_use]
    pub fn new(id: MarkerId, position: TimePosition, marker_type: MarkerType) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            position,
            duration: TimePosition::default(),
            marker_type,
            name: String::new(),
            comment: String::new(),
            color: None,
            tags: Vec::new(),
            locked: false,
            created_at: now,
            modified_at: now,
            author: None,
        }
    }

    /// Creates a chapter marker.
    #[must_use]
    pub fn chapter(id: MarkerId, position: TimePosition, name: impl Into<String>) -> Self {
        let mut marker = Self::new(id, position, MarkerType::Chapter);
        marker.name = name.into();
        marker
    }

    /// Creates a comment marker.
    #[must_use]
    pub fn with_comment(id: MarkerId, position: TimePosition, comment: impl Into<String>) -> Self {
        let mut marker = Self::new(id, position, MarkerType::Comment);
        marker.comment = comment.into();
        marker
    }

    /// Returns the marker ID.
    #[must_use]
    pub const fn id(&self) -> MarkerId {
        self.id
    }

    /// Returns the time position.
    #[must_use]
    pub const fn position(&self) -> TimePosition {
        self.position
    }

    /// Sets the time position.
    pub fn set_position(&mut self, position: TimePosition) {
        if !self.locked {
            self.position = position;
            self.modified_at = Timestamp::now();
        }
    }

    /// Returns the duration.
    #[must_use]
    pub const fn duration(&self) -> TimePosition {
        self.duration
    }

    /// Sets the duration.
    pub fn set_duration(&mut self, duration: TimePosition) {
        if !self.locked {
            self.duration = duration;
            self.modified_at = Timestamp::now();
        }
    }

    /// Returns whether this is a range marker.
    #[must_use]
    pub fn is_range(&self) -> bool {
        self.duration.ms > 0
    }

    /// Returns the end position for range markers.
    #[must_use]
    pub fn end_position(&self) -> TimePosition {
        TimePosition::from_ms(self.position.ms + self.duration.ms)
    }

    /// Returns the marker type.
    #[must_use]
    pub const fn marker_type(&self) -> MarkerType {
        self.marker_type
    }

    /// Sets the marker type.
    pub fn set_marker_type(&mut self, marker_type: MarkerType) {
        self.marker_type = marker_type;
        self.modified_at = Timestamp::now();
    }

    /// Returns the name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.modified_at = Timestamp::now();
    }

    /// Returns the comment.
    #[must_use]
    pub fn comment(&self) -> &str {
        &self.comment
    }

    /// Sets the comment.
    pub fn set_comment(&mut self, comment: impl Into<String>) {
        self.comment = comment.into();
        self.modified_at = Timestamp::now();
    }

    /// Returns the display color.
    #[must_use]
    pub fn color(&self) -> [f32; 4] {
        self.color.unwrap_or_else(|| self.marker_type.default_color())
    }

    /// Sets the color override.
    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = Some(color);
    }

    /// Clears the color override.
    pub fn clear_color(&mut self) {
        self.color = None;
    }

    /// Returns the tags.
    #[must_use]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// Adds a tag.
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Removes a tag.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    /// Returns whether the marker has a specific tag.
    #[must_use]
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Returns whether the marker is locked.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.locked
    }

    /// Sets the locked state.
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns the modification timestamp.
    #[must_use]
    pub const fn modified_at(&self) -> Timestamp {
        self.modified_at
    }

    /// Returns the author.
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Sets the author.
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.author = Some(author.into());
    }

    /// Checks if position falls within this marker's range.
    #[must_use]
    pub fn contains(&self, position: TimePosition) -> bool {
        if self.is_range() {
            position.ms >= self.position.ms && position.ms < self.end_position().ms
        } else {
            position.ms == self.position.ms
        }
    }
}

/// Filter criteria for markers.
#[derive(Debug, Clone, Default)]
pub struct MarkerFilter {
    /// Filter by marker type.
    pub marker_type:    Option<MarkerType>,
    /// Filter by tag.
    pub tag:            Option<String>,
    /// Filter by author.
    pub author:         Option<String>,
    /// Filter by time range (start).
    pub range_start:    Option<TimePosition>,
    /// Filter by time range (end).
    pub range_end:      Option<TimePosition>,
    /// Include locked markers.
    pub include_locked: bool,
    /// Search text (name or comment).
    pub search_text:    Option<String>,
}

impl MarkerFilter {
    /// Creates a new filter.
    #[must_use]
    pub fn new() -> Self {
        Self { include_locked: true, ..Default::default() }
    }

    /// Filters by marker type.
    #[must_use]
    pub fn with_type(mut self, marker_type: MarkerType) -> Self {
        self.marker_type = Some(marker_type);
        self
    }

    /// Filters by tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Filters by time range.
    #[must_use]
    pub fn with_range(mut self, start: TimePosition, end: TimePosition) -> Self {
        self.range_start = Some(start);
        self.range_end = Some(end);
        self
    }

    /// Checks if a marker matches the filter.
    #[must_use]
    pub fn matches(&self, marker: &Marker) -> bool {
        if let Some(mt) = self.marker_type
            && marker.marker_type() != mt
        {
            return false;
        }

        if let Some(tag) = &self.tag
            && !marker.has_tag(tag)
        {
            return false;
        }

        if let Some(author) = &self.author
            && marker.author() != Some(author.as_str())
        {
            return false;
        }

        if let Some(start) = self.range_start
            && marker.position().ms < start.ms
        {
            return false;
        }

        if let Some(end) = self.range_end
            && marker.position().ms > end.ms
        {
            return false;
        }

        if !self.include_locked && marker.is_locked() {
            return false;
        }

        if let Some(search) = &self.search_text {
            let search_lower = search.to_lowercase();
            let name_match = marker.name().to_lowercase().contains(&search_lower);
            let comment_match = marker.comment().to_lowercase().contains(&search_lower);
            if !name_match && !comment_match {
                return false;
            }
        }

        true
    }
}

/// Manager for timeline markers.
pub struct MarkerManager {
    /// All markers (sorted by position).
    markers:    Vec<Marker>,
    /// Next marker ID.
    next_id:    u64,
    /// Available tags (for autocomplete).
    known_tags: Vec<String>,
    /// Selection state (selected marker IDs).
    selection:  Vec<MarkerId>,
}

impl MarkerManager {
    /// Creates a new marker manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            markers:    Vec::new(),
            next_id:    1,
            known_tags: Vec::new(),
            selection:  Vec::new(),
        }
    }

    /// Generates a new marker ID.
    fn next_id(&mut self) -> MarkerId {
        let id = MarkerId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds a marker at the specified position.
    pub fn add_marker(&mut self, position: TimePosition, marker_type: MarkerType) -> MarkerId {
        let id = self.next_id();
        let marker = Marker::new(id, position, marker_type);

        // Insert in sorted order
        let pos = self
            .markers
            .iter()
            .position(|m| m.position().ms > position.ms)
            .unwrap_or(self.markers.len());
        self.markers.insert(pos, marker);

        id
    }

    /// Adds a chapter marker.
    pub fn add_chapter(&mut self, position: TimePosition, name: impl Into<String>) -> MarkerId {
        let id = self.next_id();
        let marker = Marker::chapter(id, position, name);

        let pos = self
            .markers
            .iter()
            .position(|m| m.position().ms > position.ms)
            .unwrap_or(self.markers.len());
        self.markers.insert(pos, marker);

        id
    }

    /// Removes a marker by ID.
    pub fn remove_marker(&mut self, id: MarkerId) -> bool {
        if let Some(pos) = self.markers.iter().position(|m| m.id() == id) {
            let marker = &self.markers[pos];
            if marker.is_locked() {
                return false;
            }
            self.markers.remove(pos);
            self.selection.retain(|&mid| mid != id);
            true
        } else {
            false
        }
    }

    /// Gets a marker by ID.
    #[must_use]
    pub fn get_marker(&self, id: MarkerId) -> Option<&Marker> {
        self.markers.iter().find(|m| m.id() == id)
    }

    /// Gets a mutable marker by ID.
    pub fn get_marker_mut(&mut self, id: MarkerId) -> Option<&mut Marker> {
        self.markers.iter_mut().find(|m| m.id() == id)
    }

    /// Returns all markers.
    #[must_use]
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Returns markers matching a filter.
    #[must_use]
    pub fn filter(&self, filter: &MarkerFilter) -> Vec<&Marker> {
        self.markers.iter().filter(|m| filter.matches(m)).collect()
    }

    /// Gets markers at a specific position.
    #[must_use]
    pub fn markers_at(&self, position: TimePosition) -> Vec<&Marker> {
        self.markers.iter().filter(|m| m.contains(position)).collect()
    }

    /// Gets the nearest marker to a position.
    #[must_use]
    pub fn nearest_marker(&self, position: TimePosition) -> Option<&Marker> {
        self.markers
            .iter()
            .min_by_key(|m| (m.position().ms as i64 - position.ms as i64).unsigned_abs())
    }

    /// Gets the next marker after a position.
    #[must_use]
    pub fn next_marker(&self, position: TimePosition) -> Option<&Marker> {
        self.markers.iter().find(|m| m.position().ms > position.ms)
    }

    /// Gets the previous marker before a position.
    #[must_use]
    pub fn prev_marker(&self, position: TimePosition) -> Option<&Marker> {
        self.markers.iter().rev().find(|m| m.position().ms < position.ms)
    }

    /// Gets all chapter markers.
    #[must_use]
    pub fn chapters(&self) -> Vec<&Marker> {
        self.markers.iter().filter(|m| m.marker_type() == MarkerType::Chapter).collect()
    }

    /// Moves a marker to a new position.
    pub fn move_marker(
        &mut self, id: MarkerId, new_position: TimePosition,
    ) -> VideoEditorResult<()> {
        // Find and update marker
        let marker = self
            .markers
            .iter_mut()
            .find(|m| m.id() == id)
            .ok_or_else(|| VideoEditorError::Timeline("Marker not found".into()))?;

        if marker.is_locked() {
            return Err(VideoEditorError::Timeline("Marker is locked".into()));
        }

        marker.position = new_position;
        marker.modified_at = Timestamp::now();

        // Re-sort markers
        self.markers.sort_by(|a, b| a.position().ms.cmp(&b.position().ms));

        Ok(())
    }

    /// Selects a marker.
    pub fn select(&mut self, id: MarkerId, add_to_selection: bool) {
        if !add_to_selection {
            self.selection.clear();
        }
        if !self.selection.contains(&id) {
            self.selection.push(id);
        }
    }

    /// Deselects a marker.
    pub fn deselect(&mut self, id: MarkerId) {
        self.selection.retain(|&mid| mid != id);
    }

    /// Clears selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Returns selected marker IDs.
    #[must_use]
    pub fn selection(&self) -> &[MarkerId] {
        &self.selection
    }

    /// Returns selected markers.
    #[must_use]
    pub fn selected_markers(&self) -> Vec<&Marker> {
        self.selection.iter().filter_map(|&id| self.get_marker(id)).collect()
    }

    /// Deletes selected markers.
    pub fn delete_selection(&mut self) {
        let to_delete: Vec<_> = self
            .selection
            .iter()
            .filter(|&&id| self.get_marker(id).map(|m| !m.is_locked()).unwrap_or(false))
            .copied()
            .collect();

        for id in to_delete {
            self.markers.retain(|m| m.id() != id);
        }
        self.selection.clear();
    }

    /// Registers a known tag.
    pub fn register_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.known_tags.contains(&tag) {
            self.known_tags.push(tag);
        }
    }

    /// Returns known tags.
    #[must_use]
    pub fn known_tags(&self) -> &[String] {
        &self.known_tags
    }

    /// Returns marker count.
    #[must_use]
    pub fn count(&self) -> usize {
        self.markers.len()
    }

    /// Returns chapter count.
    #[must_use]
    pub fn chapter_count(&self) -> usize {
        self.chapters().len()
    }

    /// Clears all markers.
    pub fn clear(&mut self) {
        let to_remove: Vec<_> =
            self.markers.iter().filter(|m| !m.is_locked()).map(|m| m.id()).collect();

        for id in to_remove {
            self.markers.retain(|m| m.id() != id);
        }
        self.selection.clear();
    }

    /// Creates chapters from markers.
    pub fn create_chapters_from_markers(&mut self, marker_type: MarkerType) {
        for marker in &mut self.markers {
            if marker.marker_type() == marker_type && !marker.is_locked() {
                marker.set_marker_type(MarkerType::Chapter);
            }
        }
    }
}

impl Default for MarkerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let mut manager = MarkerManager::new();
        let id = manager.add_marker(TimePosition::from_ms(1000), MarkerType::Standard);

        assert!(manager.get_marker(id).is_some());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_chapter_markers() {
        let mut manager = MarkerManager::new();
        manager.add_chapter(TimePosition::from_ms(0), "Intro");
        manager.add_chapter(TimePosition::from_ms(30000), "Chapter 1");
        manager.add_chapter(TimePosition::from_ms(60000), "Chapter 2");

        assert_eq!(manager.chapter_count(), 3);
    }

    #[test]
    fn test_marker_navigation() {
        let mut manager = MarkerManager::new();
        manager.add_marker(TimePosition::from_ms(1000), MarkerType::Standard);
        manager.add_marker(TimePosition::from_ms(2000), MarkerType::Standard);
        manager.add_marker(TimePosition::from_ms(3000), MarkerType::Standard);

        let next = manager.next_marker(TimePosition::from_ms(1500));
        assert!(next.is_some());
        assert_eq!(next.map(|m| m.position().ms), Some(2000));

        let prev = manager.prev_marker(TimePosition::from_ms(1500));
        assert!(prev.is_some());
        assert_eq!(prev.map(|m| m.position().ms), Some(1000));
    }

    #[test]
    fn test_marker_filter() {
        let mut manager = MarkerManager::new();
        manager.add_marker(TimePosition::from_ms(1000), MarkerType::Standard);
        manager.add_chapter(TimePosition::from_ms(2000), "Chapter");
        manager.add_marker(TimePosition::from_ms(3000), MarkerType::Comment);

        let filter = MarkerFilter::new().with_type(MarkerType::Chapter);
        let filtered = manager.filter(&filter);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_marker_tags() {
        let mut manager = MarkerManager::new();
        let id = manager.add_marker(TimePosition::from_ms(1000), MarkerType::Comment);

        if let Some(marker) = manager.get_marker_mut(id) {
            marker.add_tag("review");
            marker.add_tag("important");
        }

        let filter = MarkerFilter::new().with_tag("review");
        let filtered = manager.filter(&filter);
        assert_eq!(filtered.len(), 1);
    }
}

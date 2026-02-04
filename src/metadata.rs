//! Drill-down metadata types for EVLF format.
//!
//! Provides frame-level metadata including object detection,
//! scene classification, and semantic annotations.

/// Frame-level drill-down metadata.
#[derive(Debug, Clone)]
pub struct FrameMetadata {
    /// Frame number.
    pub frame_number: u64,
    /// Object detections.
    pub objects:      Vec<ObjectDetection>,
    /// Scene classification.
    pub scene:        SceneClassification,
    /// AI-generated description.
    pub description:  Option<String>,
    /// Custom annotations.
    pub annotations:  Vec<Annotation>,
    /// Semantic regions.
    pub regions:      Vec<SemanticRegion>,
}

impl FrameMetadata {
    /// Creates empty metadata for a frame.
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            objects: Vec::new(),
            scene: SceneClassification::default(),
            description: None,
            annotations: Vec::new(),
            regions: Vec::new(),
        }
    }

    /// Adds an object detection.
    pub fn add_object(&mut self, object: ObjectDetection) {
        self.objects.push(object);
    }

    /// Sets scene classification.
    pub fn set_scene(&mut self, scene: SceneClassification) {
        self.scene = scene;
    }

    /// Adds an annotation.
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }
}

/// Object detection result.
#[derive(Debug, Clone)]
pub struct ObjectDetection {
    /// Object ID (for tracking across frames).
    pub object_id:  u64,
    /// Object class.
    pub class:      String,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Bounding box.
    pub bbox:       BoundingBox,
    /// Tracking state.
    pub tracking:   Option<TrackingState>,
    /// Object attributes.
    pub attributes: Vec<(String, String)>,
}

impl ObjectDetection {
    /// Creates a new object detection.
    pub fn new(
        object_id: u64, class: impl Into<String>, confidence: f32, bbox: BoundingBox,
    ) -> Self {
        Self {
            object_id,
            class: class.into(),
            confidence,
            bbox,
            tracking: None,
            attributes: Vec::new(),
        }
    }

    /// Adds an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }

    /// Sets tracking state.
    pub fn with_tracking(mut self, tracking: TrackingState) -> Self {
        self.tracking = Some(tracking);
        self
    }
}

/// Bounding box (normalized coordinates 0.0-1.0).
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    /// Left edge (0.0-1.0).
    pub x:      f32,
    /// Top edge (0.0-1.0).
    pub y:      f32,
    /// Width (0.0-1.0).
    pub width:  f32,
    /// Height (0.0-1.0).
    pub height: f32,
}

impl BoundingBox {
    /// Creates a new bounding box.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Creates from pixel coordinates.
    pub fn from_pixels(x: u32, y: u32, w: u32, h: u32, img_w: u32, img_h: u32) -> Self {
        Self {
            x:      x as f32 / img_w as f32,
            y:      y as f32 / img_h as f32,
            width:  w as f32 / img_w as f32,
            height: h as f32 / img_h as f32,
        }
    }

    /// Converts to pixel coordinates.
    pub fn to_pixels(&self, img_w: u32, img_h: u32) -> (u32, u32, u32, u32) {
        (
            (self.x * img_w as f32) as u32,
            (self.y * img_h as f32) as u32,
            (self.width * img_w as f32) as u32,
            (self.height * img_h as f32) as u32,
        )
    }

    /// Calculates center point.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Calculates area.
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Calculates IoU with another box.
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
        let union = self.area() + other.area() - intersection;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }
}

/// Object tracking state.
#[derive(Debug, Clone, Copy)]
pub struct TrackingState {
    /// Velocity (normalized units per frame).
    pub velocity:       (f32, f32),
    /// Frames tracked.
    pub tracked_frames: u32,
    /// First appearance frame.
    pub first_frame:    u64,
    /// Last seen frame.
    pub last_frame:     u64,
}

impl TrackingState {
    /// Creates a new tracking state.
    pub fn new(first_frame: u64) -> Self {
        Self { velocity: (0.0, 0.0), tracked_frames: 1, first_frame, last_frame: first_frame }
    }

    /// Updates tracking with new position.
    pub fn update(&mut self, current_frame: u64, prev_center: (f32, f32), curr_center: (f32, f32)) {
        let dt = (current_frame - self.last_frame) as f32;
        if dt > 0.0 {
            self.velocity = (
                (curr_center.0 - prev_center.0) / dt,
                (curr_center.1 - prev_center.1) / dt,
            );
        }
        self.tracked_frames += 1;
        self.last_frame = current_frame;
    }
}

/// Scene classification.
#[derive(Debug, Clone, Default)]
pub struct SceneClassification {
    /// Primary scene type.
    pub primary:      String,
    /// Confidence.
    pub confidence:   f32,
    /// Alternative classifications.
    pub alternatives: Vec<(String, f32)>,
    /// Scene attributes.
    pub attributes:   Vec<String>,
}

impl SceneClassification {
    /// Creates a new scene classification.
    pub fn new(primary: impl Into<String>, confidence: f32) -> Self {
        Self {
            primary: primary.into(),
            confidence,
            alternatives: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Adds an alternative classification.
    pub fn with_alternative(mut self, class: impl Into<String>, confidence: f32) -> Self {
        self.alternatives.push((class.into(), confidence));
        self
    }

    /// Adds a scene attribute.
    pub fn with_attribute(mut self, attribute: impl Into<String>) -> Self {
        self.attributes.push(attribute.into());
        self
    }
}

/// Custom annotation.
#[derive(Debug, Clone)]
pub struct Annotation {
    /// Annotation type.
    pub annotation_type: AnnotationType,
    /// Annotation value.
    pub value:           String,
    /// Optional region.
    pub region:          Option<BoundingBox>,
    /// Author/source.
    pub author:          String,
    /// Timestamp (ms from start).
    pub timestamp_ms:    u64,
}

/// Annotation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotationType {
    /// Text note.
    Note,
    /// Label/tag.
    Label,
    /// Marker.
    Marker,
    /// Review comment.
    Comment,
    /// AI-generated.
    AIGenerated,
    /// Quality issue.
    Issue,
    /// Approval.
    Approval,
}

/// Semantic region in frame.
#[derive(Debug, Clone)]
pub struct SemanticRegion {
    /// Region ID.
    pub region_id:   u32,
    /// Region type (sky, ground, water, building, etc.).
    pub region_type: String,
    /// Polygon vertices (normalized 0.0-1.0).
    pub polygon:     Vec<(f32, f32)>,
    /// Depth estimate (relative, 0.0 near - 1.0 far).
    pub depth:       f32,
}

impl SemanticRegion {
    /// Creates a new semantic region.
    pub fn new(region_id: u32, region_type: impl Into<String>) -> Self {
        Self { region_id, region_type: region_type.into(), polygon: Vec::new(), depth: 0.5 }
    }

    /// Adds a polygon vertex.
    pub fn add_vertex(&mut self, x: f32, y: f32) {
        self.polygon.push((x, y));
    }

    /// Sets depth estimate.
    pub fn with_depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }
}

/// Metadata index for efficient lookup.
#[derive(Debug, Clone, Default)]
pub struct MetadataIndex {
    /// Frame-to-offset mapping.
    frame_offsets:     Vec<(u64, u64)>,
    /// Object tracking index (object_id -> frames).
    object_index:      Vec<(u64, Vec<u64>)>,
    /// Scene transition frames.
    scene_transitions: Vec<(u64, String)>,
}

impl MetadataIndex {
    /// Creates a new metadata index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a frame offset.
    pub fn add_frame(&mut self, frame: u64, offset: u64) {
        self.frame_offsets.push((frame, offset));
    }

    /// Adds object tracking entry.
    pub fn track_object(&mut self, object_id: u64, frame: u64) {
        for (id, frames) in &mut self.object_index {
            if *id == object_id {
                frames.push(frame);
                return;
            }
        }
        self.object_index.push((object_id, vec![frame]));
    }

    /// Adds scene transition.
    pub fn add_scene_transition(&mut self, frame: u64, scene: impl Into<String>) {
        self.scene_transitions.push((frame, scene.into()));
    }

    /// Finds frames containing object.
    pub fn frames_with_object(&self, object_id: u64) -> Option<&[u64]> {
        self.object_index
            .iter()
            .find(|(id, _)| *id == object_id)
            .map(|(_, frames)| frames.as_slice())
    }

    /// Gets scene transitions.
    pub fn scene_transitions(&self) -> &[(u64, String)] {
        &self.scene_transitions
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box_iou() {
        let box1 = BoundingBox::new(0.0, 0.0, 0.5, 0.5);
        let box2 = BoundingBox::new(0.25, 0.25, 0.5, 0.5);

        let iou = box1.iou(&box2);
        // Intersection = 0.25 * 0.25 = 0.0625
        // Union = 0.25 + 0.25 - 0.0625 = 0.4375
        // IoU = 0.0625 / 0.4375 ≈ 0.143
        assert!(iou > 0.14 && iou < 0.15);
    }

    #[test]
    fn test_tracking_state() {
        let mut state = TrackingState::new(0);
        state.update(1, (0.5, 0.5), (0.6, 0.5));

        assert_eq!(state.tracked_frames, 2);
        assert!((state.velocity.0 - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_metadata_index() {
        let mut index = MetadataIndex::new();
        index.track_object(1, 0);
        index.track_object(1, 1);
        index.track_object(2, 1);

        let frames = index.frames_with_object(1).expect("test assertion");
        assert_eq!(frames.len(), 2);
    }
}

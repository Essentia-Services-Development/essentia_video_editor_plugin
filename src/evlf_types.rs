//! EVLF (Essentia Video Layered Format) types.
//!
//! This module defines the binary format for multi-layer video containers
//! with drill-down metadata and next-frame branching capabilities.

#![allow(unused_imports)]

use core::mem::size_of;

/// EVLF magic number: "EVLF" in big-endian.
pub const EVLF_MAGIC: u32 = 0x45564C46;

/// Current format version (1.0.0).
pub const EVLF_VERSION: u32 = 0x00010000;

/// Trailer magic: "FLVE" (reverse).
pub const EVLF_TRAILER_MAGIC: u32 = 0x464C5645;

/// Header size in bytes.
pub const EVLF_HEADER_SIZE: usize = 64;

/// EVLF container header (64 bytes).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct EvlfHeader {
    /// Magic number (0x45564C46).
    pub magic:           u32,
    /// Format version.
    pub version:         u32,
    /// Container flags.
    pub flags:           u32,
    /// Number of tracks.
    pub track_count:     u32,
    /// Total frame count.
    pub frame_count:     u64,
    /// Duration in milliseconds.
    pub duration_ms:     u64,
    /// Video width.
    pub width:           u32,
    /// Video height.
    pub height:          u32,
    /// Frame rate numerator.
    pub frame_rate_num:  u32,
    /// Frame rate denominator.
    pub frame_rate_den:  u32,
    /// Metadata section offset.
    pub metadata_offset: u64,
    /// Frame index offset.
    pub index_offset:    u64,
}

impl EvlfHeader {
    /// Creates a new header with default values.
    pub fn new(width: u32, height: u32, frame_rate_num: u32, frame_rate_den: u32) -> Self {
        Self {
            magic: EVLF_MAGIC,
            version: EVLF_VERSION,
            flags: 0,
            track_count: 0,
            frame_count: 0,
            duration_ms: 0,
            width,
            height,
            frame_rate_num,
            frame_rate_den,
            metadata_offset: 0,
            index_offset: 0,
        }
    }

    /// Validates the header magic number.
    pub fn is_valid(&self) -> bool {
        self.magic == EVLF_MAGIC
    }

    /// Converts to bytes for writing.
    pub fn to_bytes(&self) -> [u8; EVLF_HEADER_SIZE] {
        let mut bytes = [0u8; EVLF_HEADER_SIZE];
        let mut offset = 0;

        Self::write_u32(&mut bytes, &mut offset, self.magic);
        Self::write_u32(&mut bytes, &mut offset, self.version);
        Self::write_u32(&mut bytes, &mut offset, self.flags);
        Self::write_u32(&mut bytes, &mut offset, self.track_count);
        Self::write_u64(&mut bytes, &mut offset, self.frame_count);
        Self::write_u64(&mut bytes, &mut offset, self.duration_ms);
        Self::write_u32(&mut bytes, &mut offset, self.width);
        Self::write_u32(&mut bytes, &mut offset, self.height);
        Self::write_u32(&mut bytes, &mut offset, self.frame_rate_num);
        Self::write_u32(&mut bytes, &mut offset, self.frame_rate_den);
        Self::write_u64(&mut bytes, &mut offset, self.metadata_offset);
        Self::write_u64(&mut bytes, &mut offset, self.index_offset);

        bytes
    }

    /// Parses header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < EVLF_HEADER_SIZE {
            return None;
        }

        let mut offset = 0;
        Some(Self {
            magic:           Self::read_u32(bytes, &mut offset),
            version:         Self::read_u32(bytes, &mut offset),
            flags:           Self::read_u32(bytes, &mut offset),
            track_count:     Self::read_u32(bytes, &mut offset),
            frame_count:     Self::read_u64(bytes, &mut offset),
            duration_ms:     Self::read_u64(bytes, &mut offset),
            width:           Self::read_u32(bytes, &mut offset),
            height:          Self::read_u32(bytes, &mut offset),
            frame_rate_num:  Self::read_u32(bytes, &mut offset),
            frame_rate_den:  Self::read_u32(bytes, &mut offset),
            metadata_offset: Self::read_u64(bytes, &mut offset),
            index_offset:    Self::read_u64(bytes, &mut offset),
        })
    }

    fn write_u32(bytes: &mut [u8], offset: &mut usize, value: u32) {
        bytes[*offset..*offset + 4].copy_from_slice(&value.to_le_bytes());
        *offset += 4;
    }

    fn write_u64(bytes: &mut [u8], offset: &mut usize, value: u64) {
        bytes[*offset..*offset + 8].copy_from_slice(&value.to_le_bytes());
        *offset += 8;
    }

    fn read_u32(bytes: &[u8], offset: &mut usize) -> u32 {
        let value = u32::from_le_bytes([
            bytes[*offset],
            bytes[*offset + 1],
            bytes[*offset + 2],
            bytes[*offset + 3],
        ]);
        *offset += 4;
        value
    }

    fn read_u64(bytes: &[u8], offset: &mut usize) -> u64 {
        let value = u64::from_le_bytes([
            bytes[*offset],
            bytes[*offset + 1],
            bytes[*offset + 2],
            bytes[*offset + 3],
            bytes[*offset + 4],
            bytes[*offset + 5],
            bytes[*offset + 6],
            bytes[*offset + 7],
        ]);
        *offset += 8;
        value
    }
}

/// Container flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct EvlfFlags(pub u32);

impl EvlfFlags {
    /// Has alpha channel.
    pub const HAS_ALPHA: u32 = 1 << 0;
    /// Has audio tracks.
    pub const HAS_AUDIO: u32 = 1 << 1;
    /// Has branching/interactivity.
    pub const HAS_BRANCHES: u32 = 1 << 2;
    /// Has drill-down metadata.
    pub const HAS_METADATA: u32 = 1 << 3;
    /// Uses GPU-accelerated codec.
    pub const GPU_ACCELERATED: u32 = 1 << 4;
    /// HDR content.
    pub const HDR_CONTENT: u32 = 1 << 5;
    /// 3D stereoscopic.
    pub const STEREOSCOPIC_3D: u32 = 1 << 6;
    /// Contains AI annotations.
    pub const AI_ANNOTATED: u32 = 1 << 7;

    /// Creates new flags.
    pub fn new() -> Self {
        Self(0)
    }

    /// Sets a flag.
    pub fn set(&mut self, flag: u32) {
        self.0 |= flag;
    }

    /// Clears a flag.
    pub fn clear(&mut self, flag: u32) {
        self.0 &= !flag;
    }

    /// Checks if flag is set.
    pub fn has(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
}

/// Track type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum EvlfTrackType {
    /// Primary video stream.
    #[default]
    Video       = 0,
    /// Audio stream.
    Audio       = 1,
    /// Text/subtitle overlay.
    Text        = 2,
    /// Effect/filter layer.
    Effect      = 3,
    /// 3D geometry layer.
    Geometry3D  = 4,
    /// Vector graphics layer.
    Vector      = 5,
    /// Particle system layer.
    Particles   = 6,
    /// AI-generated content layer.
    AIContent   = 7,
    /// Interactive hotspot layer.
    Interactive = 8,
    /// Metadata-only track.
    Metadata    = 255,
}

/// Track flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrackFlags(pub u8);

impl TrackFlags {
    /// Track is enabled.
    pub const ENABLED: u8 = 1 << 0;
    /// Track is muted.
    pub const MUTED: u8 = 1 << 1;
    /// Track is locked.
    pub const LOCKED: u8 = 1 << 2;
    /// Track is soloed.
    pub const SOLO: u8 = 1 << 3;

    /// Creates enabled track flags.
    pub fn enabled() -> Self {
        Self(Self::ENABLED)
    }
}

/// Blend mode for layer compositing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BlendMode {
    #[default]
    /// Normal blending mode - no blending applied
    Normal     = 0,
    /// Multiply blending mode - multiplies base and blend colors
    Multiply   = 1,
    /// Screen blending mode - inverts, multiplies, then inverts again
    Screen     = 2,
    /// Overlay blending mode - combines multiply and screen based on base color
    Overlay    = 3,
    /// Darken blending mode - selects the darker of base and blend colors
    Darken     = 4,
    /// Lighten blending mode - selects the lighter of base and blend colors
    Lighten    = 5,
    /// Color Dodge blending mode - brightens base color based on blend color
    ColorDodge = 6,
    /// Color Burn blending mode - darkens base color based on blend color
    ColorBurn  = 7,
    /// Hard Light blending mode - like overlay but uses blend color as base
    HardLight  = 8,
    /// Soft Light blending mode - softer version of hard light
    SoftLight  = 9,
    /// Difference blending mode - absolute difference between colors
    Difference = 10,
    /// Exclusion blending mode - similar to difference but lower contrast
    Exclusion  = 11,
    /// Hue blending mode - uses hue from blend color with saturation/luminance
    /// from base
    Hue        = 12,
    /// Saturation blending mode - uses saturation from blend color with
    /// hue/luminance from base
    Saturation = 13,
    /// Color blending mode - uses hue and saturation from blend color with
    /// luminance from base
    Color      = 14,
    /// Luminosity blending mode - uses luminance from blend color with
    /// hue/saturation from base
    Luminosity = 15,
    /// Add blending mode - adds base and blend colors
    Add        = 16,
    /// Subtract blending mode - subtracts blend color from base color
    Subtract   = 17,
}

/// Track header (96 bytes).
#[derive(Debug, Clone)]
pub struct EvlfTrackHeader {
    /// Track ID.
    pub track_id:    u32,
    /// Track type.
    pub track_type:  EvlfTrackType,
    /// Track flags.
    pub flags:       TrackFlags,
    /// Track name.
    pub name:        String,
    /// Codec identifier (fourcc).
    pub codec:       u32,
    /// Layer Z-order (0 = bottom).
    pub z_order:     u32,
    /// Blend mode.
    pub blend_mode:  BlendMode,
    /// Opacity (0-255).
    pub opacity:     u8,
    /// Track data offset.
    pub data_offset: u64,
    /// Track data size.
    pub data_size:   u64,
}

impl EvlfTrackHeader {
    /// Creates a new video track header.
    pub fn video(track_id: u32, name: impl Into<String>) -> Self {
        Self {
            track_id,
            track_type: EvlfTrackType::Video,
            flags: TrackFlags::enabled(),
            name: name.into(),
            codec: 0x68323634, // "h264"
            z_order: 0,
            blend_mode: BlendMode::Normal,
            opacity: 255,
            data_offset: 0,
            data_size: 0,
        }
    }

    /// Creates a new audio track header.
    pub fn audio(track_id: u32, name: impl Into<String>) -> Self {
        Self {
            track_id,
            track_type: EvlfTrackType::Audio,
            flags: TrackFlags::enabled(),
            name: name.into(),
            codec: 0x61616320, // "aac "
            z_order: 0,
            blend_mode: BlendMode::Normal,
            opacity: 255,
            data_offset: 0,
            data_size: 0,
        }
    }
}

/// Frame type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FrameType {
    /// Keyframe (I-frame).
    #[default]
    Keyframe      = 0,
    /// Predictive frame (P-frame).
    Predictive    = 1,
    /// Bidirectional frame (B-frame).
    Bidirectional = 2,
    /// Branch point frame.
    BranchPoint   = 3,
    /// Merge point frame.
    MergePoint    = 4,
}

/// Frame index entry (48 bytes).
#[derive(Debug, Clone, Copy)]
pub struct FrameIndexEntry {
    /// Frame number (0-indexed).
    pub frame_number:    u64,
    /// Presentation timestamp (ms).
    pub pts_ms:          u64,
    /// Decode timestamp (ms).
    pub dts_ms:          u64,
    /// Frame type.
    pub frame_type:      FrameType,
    /// Frame data offset.
    pub data_offset:     u64,
    /// Frame data size.
    pub data_size:       u32,
    /// Branch point ID (0 = none).
    pub branch_id:       u32,
    /// Metadata offset (0 = none).
    pub metadata_offset: u64,
}

impl FrameIndexEntry {
    /// Creates a new keyframe entry.
    pub fn keyframe(frame_number: u64, pts_ms: u64, data_offset: u64, data_size: u32) -> Self {
        Self {
            frame_number,
            pts_ms,
            dts_ms: pts_ms,
            frame_type: FrameType::Keyframe,
            data_offset,
            data_size,
            branch_id: 0,
            metadata_offset: 0,
        }
    }
}

/// Branch type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BranchType {
    /// User choice (interactive).
    #[default]
    UserChoice   = 0,
    /// AI decision (agentic).
    AIDecision   = 1,
    /// Conditional (expression-based).
    Conditional  = 2,
    /// Random (A/B testing).
    Random       = 3,
    /// Time-based (scheduled).
    TimeBased    = 4,
    /// Context-aware (based on metadata).
    ContextAware = 5,
}

/// Branch fork definition.
#[derive(Debug, Clone)]
pub struct BranchFork {
    /// Fork identifier.
    pub fork_id:      u32,
    /// Target frame number.
    pub target_frame: u64,
    /// Fork label.
    pub label:        String,
    /// Fork weight (for random/AI selection).
    pub weight:       f32,
}

/// Branch point for next-frame forking.
#[derive(Debug, Clone)]
pub struct BranchPoint {
    /// Branch ID.
    pub branch_id:    u32,
    /// Frame number where branch occurs.
    pub frame_number: u64,
    /// Branch type.
    pub branch_type:  BranchType,
    /// Available forks.
    pub forks:        Vec<BranchFork>,
}

impl BranchPoint {
    /// Creates a new user-choice branch point.
    pub fn user_choice(branch_id: u32, frame_number: u64, forks: Vec<BranchFork>) -> Self {
        Self { branch_id, frame_number, branch_type: BranchType::UserChoice, forks }
    }

    /// Creates a new AI-decision branch point.
    pub fn ai_decision(branch_id: u32, frame_number: u64, forks: Vec<BranchFork>) -> Self {
        Self { branch_id, frame_number, branch_type: BranchType::AIDecision, forks }
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_header_size() {
        // Verify header fits in expected size
        assert!(u32::BITS as usize + size_of::<u64>() * 4 <= EVLF_HEADER_SIZE);
    }

    #[test]
    fn test_header_roundtrip() {
        let header = EvlfHeader::new(1920, 1080, 30, 1);
        let bytes = header.to_bytes();
        let parsed = EvlfHeader::from_bytes(&bytes).expect("test assertion");

        assert_eq!(parsed.magic, EVLF_MAGIC);
        assert_eq!(parsed.width, 1920);
        assert_eq!(parsed.height, 1080);
        assert_eq!(parsed.frame_rate_num, 30);
    }

    #[test]
    fn test_flags() {
        let mut flags = EvlfFlags::new();
        assert!(!flags.has(EvlfFlags::HAS_ALPHA));

        flags.set(EvlfFlags::HAS_ALPHA);
        flags.set(EvlfFlags::HAS_METADATA);

        assert!(flags.has(EvlfFlags::HAS_ALPHA));
        assert!(flags.has(EvlfFlags::HAS_METADATA));
        assert!(!flags.has(EvlfFlags::HAS_AUDIO));

        flags.clear(EvlfFlags::HAS_ALPHA);
        assert!(!flags.has(EvlfFlags::HAS_ALPHA));
    }

    #[test]
    fn test_branch_point() {
        let forks = vec![
            BranchFork {
                fork_id:      1,
                target_frame: 100,
                label:        "Action".into(),
                weight:       0.5,
            },
            BranchFork {
                fork_id:      2,
                target_frame: 200,
                label:        "Drama".into(),
                weight:       0.5,
            },
        ];

        let branch = BranchPoint::user_choice(1, 50, forks);
        assert_eq!(branch.forks.len(), 2);
        assert_eq!(branch.frame_number, 50);
    }
}

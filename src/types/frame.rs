//! Frame types for video processing pipeline.
//!
//! Inspired by rust-av's Frame and media-rs's FrameData abstractions.

use std::sync::Arc;

use super::{
    color::Formaton,
    core::{FrameRate, Resolution, TimeInfo},
};

/// Frame type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FrameType {
    /// Intra frame (keyframe) - can be decoded independently.
    Intra,
    /// Inter frame - requires previous frames for decoding.
    Inter,
    /// Bidirectional frame - requires both past and future frames.
    Bidirectional,
    /// Skip frame - use previous frame content.
    Skip,
    /// Unknown frame type.
    #[default]
    Unknown,
}

/// Plane descriptor for multi-planar frame formats.
#[derive(Debug, Clone, Copy)]
pub enum PlaneDescriptor {
    /// Video plane with stride and height.
    Video {
        /// Bytes per row.
        stride: usize,
        /// Number of rows.
        height: u32,
    },
    /// Audio plane with sample count.
    Audio {
        /// Number of samples.
        samples: usize,
    },
}

impl PlaneDescriptor {
    /// Creates a video plane descriptor.
    #[must_use]
    pub const fn video(stride: usize, height: u32) -> Self {
        Self::Video { stride, height }
    }

    /// Creates an audio plane descriptor.
    #[must_use]
    pub const fn audio(samples: usize) -> Self {
        Self::Audio { samples }
    }

    /// Returns the byte size of this plane.
    #[must_use]
    pub const fn byte_size(&self) -> usize {
        match self {
            Self::Video { stride, height } => *stride * (*height as usize),
            Self::Audio { samples } => *samples * 4, // Assuming f32 samples
        }
    }
}

/// Frame buffer error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// Invalid plane index.
    InvalidPlaneIndex,
    /// Invalid buffer size.
    InvalidBufferSize,
    /// Format mismatch.
    FormatMismatch,
    /// Buffer allocation failed.
    AllocationFailed,
    /// Buffer is locked.
    BufferLocked,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPlaneIndex => write!(f, "Invalid plane index"),
            Self::InvalidBufferSize => write!(f, "Invalid buffer size"),
            Self::FormatMismatch => write!(f, "Format mismatch"),
            Self::AllocationFailed => write!(f, "Buffer allocation failed"),
            Self::BufferLocked => write!(f, "Buffer is locked"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Video stream information.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    /// Frame width.
    pub width:      usize,
    /// Frame height.
    pub height:     usize,
    /// Frame is stored upside down.
    pub flipped:    bool,
    /// Frame type (I/P/B).
    pub frame_type: FrameType,
    /// Pixel format.
    pub format:     Arc<Formaton>,
    /// Bits per sample.
    pub bit_depth:  u8,
    /// Frame rate.
    pub frame_rate: FrameRate,
}

impl VideoInfo {
    /// Creates a new `VideoInfo`.
    #[must_use]
    pub fn new(width: usize, height: usize, format: Arc<Formaton>, frame_type: FrameType) -> Self {
        let bit_depth = format.total_depth();
        Self {
            width,
            height,
            flipped: false,
            frame_type,
            format,
            bit_depth,
            frame_rate: FrameRate::default(),
        }
    }

    /// Returns the resolution.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        Resolution::new(self.width as u32, self.height as u32)
    }

    /// Returns the total number of pixels.
    #[must_use]
    pub const fn pixel_count(&self) -> usize {
        self.width * self.height
    }

    /// Calculates required buffer size for this format.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        let bits = self.format.total_depth() as usize;
        (self.width * self.height * bits).div_ceil(8)
    }
}

/// Audio stream information.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioInfo {
    /// Number of samples per channel.
    pub samples:     usize,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels:    u8,
    /// Bits per sample.
    pub bit_depth:   u8,
    /// Audio is planar (channels in separate planes).
    pub planar:      bool,
}

impl AudioInfo {
    /// Creates a new `AudioInfo`.
    #[must_use]
    pub const fn new(samples: usize, sample_rate: u32, channels: u8, bit_depth: u8) -> Self {
        Self { samples, sample_rate, channels, bit_depth, planar: false }
    }

    /// Returns duration in milliseconds.
    #[must_use]
    pub fn duration_ms(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        (self.samples as f64 * 1000.0) / self.sample_rate as f64
    }

    /// Calculates required buffer size.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        let bytes_per_sample = (self.bit_depth as usize).div_ceil(8);
        self.samples * self.channels as usize * bytes_per_sample
    }
}

/// Frame data container with multi-plane support.
#[derive(Clone)]
pub struct FrameData {
    /// Raw data storage.
    data:    Vec<u8>,
    /// Plane descriptors.
    planes:  Vec<PlaneDescriptor>,
    /// Plane offsets within data.
    offsets: Vec<usize>,
}

impl FrameData {
    /// Creates a new empty frame data.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new(), planes: Vec::new(), offsets: Vec::new() }
    }

    /// Creates frame data with pre-allocated capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize, plane_count: usize) -> Self {
        Self {
            data:    Vec::with_capacity(capacity),
            planes:  Vec::with_capacity(plane_count),
            offsets: Vec::with_capacity(plane_count),
        }
    }

    /// Allocates planes for the specified video info.
    pub fn allocate_video_planes(&mut self, info: &VideoInfo) -> Result<(), FrameError> {
        self.planes.clear();
        self.offsets.clear();

        let mut total_size = 0usize;

        // Calculate plane sizes based on format
        for i in 0..info.format.component_count() {
            if let Some(chromaton) = info.format.component(i) {
                let (h_ss, v_ss) = chromaton.subsampling();
                let plane_width = if h_ss > 0 {
                    info.width >> h_ss
                } else {
                    info.width
                };
                let plane_height = if v_ss > 0 {
                    info.height >> v_ss
                } else {
                    info.height
                };

                let stride = Self::align_stride(plane_width, 32); // 32-byte alignment
                let plane_size = stride * plane_height;

                self.offsets.push(total_size);
                self.planes.push(PlaneDescriptor::video(stride, plane_height as u32));
                total_size += plane_size;
            }
        }

        // Allocate the data buffer
        self.data.resize(total_size, 0);
        Ok(())
    }

    /// Returns plane count.
    #[must_use]
    pub fn plane_count(&self) -> usize {
        self.planes.len()
    }

    /// Returns a reference to plane data.
    pub fn plane(&self, index: usize) -> Result<&[u8], FrameError> {
        if index >= self.planes.len() {
            return Err(FrameError::InvalidPlaneIndex);
        }

        let offset = self.offsets[index];
        let size = self.planes[index].byte_size();
        Ok(&self.data[offset..offset + size])
    }

    /// Returns a mutable reference to plane data.
    pub fn plane_mut(&mut self, index: usize) -> Result<&mut [u8], FrameError> {
        if index >= self.planes.len() {
            return Err(FrameError::InvalidPlaneIndex);
        }

        let offset = self.offsets[index];
        let size = self.planes[index].byte_size();
        Ok(&mut self.data[offset..offset + size])
    }

    /// Returns plane stride (bytes per row).
    #[must_use]
    pub fn plane_stride(&self, index: usize) -> Option<usize> {
        self.planes.get(index).and_then(|p| match p {
            PlaneDescriptor::Video { stride, .. } => Some(*stride),
            _ => None,
        })
    }

    /// Aligns stride to the specified alignment.
    const fn align_stride(width: usize, alignment: usize) -> usize {
        (width + alignment - 1) & !(alignment - 1)
    }
}

impl Default for FrameData {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for FrameData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameData")
            .field("data_size", &self.data.len())
            .field("plane_count", &self.planes.len())
            .finish()
    }
}

/// Video frame with data and metadata.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Video stream information.
    pub info:      VideoInfo,
    /// Frame data.
    pub data:      FrameData,
    /// Timestamp information.
    pub time_info: TimeInfo,
    /// Frame sequence number.
    pub sequence:  u64,
}

impl VideoFrame {
    /// Creates a new video frame with allocated buffers.
    pub fn new(info: VideoInfo) -> Result<Self, FrameError> {
        let mut data = FrameData::new();
        data.allocate_video_planes(&info)?;

        Ok(Self { info, data, time_info: TimeInfo::default(), sequence: 0 })
    }

    /// Creates a video frame from existing data.
    #[must_use]
    pub fn from_data(info: VideoInfo, data: FrameData, time_info: TimeInfo) -> Self {
        Self { info, data, time_info, sequence: 0 }
    }

    /// Returns the frame resolution.
    #[must_use]
    pub fn resolution(&self) -> Resolution {
        self.info.resolution()
    }

    /// Returns the frame type.
    #[must_use]
    pub const fn frame_type(&self) -> FrameType {
        self.info.frame_type
    }

    /// Checks if this is a keyframe.
    #[must_use]
    pub const fn is_keyframe(&self) -> bool {
        matches!(self.info.frame_type, FrameType::Intra)
    }
}

/// Frame buffer for GPU-compatible memory management.
pub struct FrameBuffer {
    /// Underlying data.
    data:     Vec<u8>,
    /// Buffer size.
    size:     usize,
    /// Reference count.
    refcount: std::sync::atomic::AtomicU32,
}

impl FrameBuffer {
    /// Creates a new frame buffer with the specified size.
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self { data: vec![0u8; size], size, refcount: std::sync::atomic::AtomicU32::new(1) }
    }

    /// Returns the buffer size.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// Returns a slice of the buffer data.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable slice of the buffer data.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Increments the reference count.
    pub fn retain(&self) {
        self.refcount.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Decrements the reference count and returns true if it reached zero.
    pub fn release(&self) -> bool {
        self.refcount.fetch_sub(1, std::sync::atomic::Ordering::Release) == 1
    }
}

/// Frame buffer pool for efficient frame reuse.
pub struct FrameBufferPool {
    /// Available buffers.
    buffers:     Vec<Arc<FrameBuffer>>,
    /// Buffer size for this pool.
    buffer_size: usize,
    /// Maximum pool size.
    max_size:    usize,
}

impl FrameBufferPool {
    /// Creates a new frame buffer pool.
    #[must_use]
    pub fn new(buffer_size: usize, max_size: usize) -> Self {
        Self { buffers: Vec::with_capacity(max_size), buffer_size, max_size }
    }

    /// Acquires a buffer from the pool or creates a new one.
    pub fn acquire(&mut self) -> Arc<FrameBuffer> {
        if let Some(buffer) = self.buffers.pop() {
            buffer
        } else {
            Arc::new(FrameBuffer::new(self.buffer_size))
        }
    }

    /// Returns a buffer to the pool.
    pub fn release(&mut self, buffer: Arc<FrameBuffer>) {
        if self.buffers.len() < self.max_size {
            self.buffers.push(buffer);
        }
    }

    /// Clears all pooled buffers.
    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    /// Returns the current number of pooled buffers.
    #[must_use]
    pub fn pooled_count(&self) -> usize {
        self.buffers.len()
    }
}

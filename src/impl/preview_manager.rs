//! Preview Manager for Essentia Video Editor Plugin
//! GAP-220-B-004: Real-time Preview System
//!
//! Features: Playback control, scrubbing, proxy preview,
//! frame caching, multi-resolution preview, and real-time monitoring.

use crate::{
    errors::VideoEditorResult,
    types::{FrameRate, Resolution, TimePosition},
};

/// Playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlaybackState {
    /// Stopped (at start).
    #[default]
    Stopped,
    /// Playing forward.
    Playing,
    /// Paused at current position.
    Paused,
    /// Scrubbing (user dragging playhead).
    Scrubbing,
    /// Shuttle forward.
    ShuttleForward,
    /// Shuttle backward.
    ShuttleBackward,
    /// Frame-by-frame stepping.
    Stepping,
    /// Rendering (non-real-time preview).
    Rendering,
}

/// Playback speed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackSpeed(f64);

impl PlaybackSpeed {
    /// Normal speed (1x).
    pub const NORMAL: Self = Self(1.0);
    /// Half speed (0.5x).
    pub const HALF: Self = Self(0.5);
    /// Double speed (2x).
    pub const DOUBLE: Self = Self(2.0);
    /// Slow motion (0.25x).
    pub const SLOW_MO: Self = Self(0.25);
    /// Fast forward (4x).
    pub const FAST_4X: Self = Self(4.0);
    /// Fast forward (8x).
    pub const FAST_8X: Self = Self(8.0);

    /// Creates a new playback speed.
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self(speed.clamp(-16.0, 16.0))
    }

    /// Returns the speed value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Returns whether playing in reverse.
    #[must_use]
    pub fn is_reverse(&self) -> bool {
        self.0 < 0.0
    }

    /// Returns the absolute speed.
    #[must_use]
    pub fn abs(&self) -> f64 {
        self.0.abs()
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// Preview quality setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PreviewQuality {
    /// Full resolution.
    Full,
    /// Half resolution.
    #[default]
    Half,
    /// Quarter resolution.
    Quarter,
    /// Eighth resolution.
    Eighth,
    /// Auto (adaptive based on performance).
    Auto,
    /// Draft (fastest, lowest quality).
    Draft,
}

impl PreviewQuality {
    /// Returns the scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Half => 0.5,
            Self::Quarter => 0.25,
            Self::Eighth => 0.125,
            Self::Auto => 1.0, // Determined at runtime
            Self::Draft => 0.125,
        }
    }

    /// Calculates the preview resolution.
    #[must_use]
    pub fn calculate_resolution(&self, source: Resolution) -> Resolution {
        let scale = self.scale_factor();
        Resolution::new(
            ((source.width as f32 * scale) as u32).max(1),
            ((source.height as f32 * scale) as u32).max(1),
        )
    }
}

/// Loop mode for playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LoopMode {
    /// No looping.
    #[default]
    None,
    /// Loop entire timeline.
    All,
    /// Loop in/out range.
    InOut,
    /// Ping-pong (forward then reverse).
    PingPong,
}

/// In/out point markers for range playback.
#[derive(Debug, Clone, Copy, Default)]
pub struct InOutPoints {
    /// In point.
    pub in_point:  Option<TimePosition>,
    /// Out point.
    pub out_point: Option<TimePosition>,
    /// Whether range is active.
    pub active:    bool,
}

impl InOutPoints {
    /// Creates new in/out points.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the in point.
    pub fn set_in(&mut self, position: TimePosition) {
        self.in_point = Some(position);
    }

    /// Sets the out point.
    pub fn set_out(&mut self, position: TimePosition) {
        self.out_point = Some(position);
    }

    /// Clears the in point.
    pub fn clear_in(&mut self) {
        self.in_point = None;
    }

    /// Clears the out point.
    pub fn clear_out(&mut self) {
        self.out_point = None;
    }

    /// Clears both points.
    pub fn clear_all(&mut self) {
        self.in_point = None;
        self.out_point = None;
        self.active = false;
    }

    /// Returns the duration of the range.
    #[must_use]
    pub fn duration(&self) -> Option<TimePosition> {
        match (self.in_point, self.out_point) {
            (Some(in_p), Some(out_p)) if out_p.ms > in_p.ms => {
                Some(TimePosition::from_ms(out_p.ms - in_p.ms))
            },
            _ => None,
        }
    }

    /// Checks if a position is within the range.
    #[must_use]
    pub fn contains(&self, position: TimePosition) -> bool {
        let in_ok = self.in_point.is_none_or(|p| position.ms >= p.ms);
        let out_ok = self.out_point.is_none_or(|p| position.ms <= p.ms);
        in_ok && out_ok
    }
}

/// Frame cache for preview performance.
#[derive(Debug)]
pub struct FrameCache {
    /// Maximum cache size in bytes.
    max_size:     usize,
    /// Current cache size in bytes.
    current_size: usize,
    /// Cached frame entries.
    entries:      Vec<CachedFrame>,
    /// Cache hit count.
    hits:         u64,
    /// Cache miss count.
    misses:       u64,
}

/// A cached frame.
#[derive(Debug)]
pub struct CachedFrame {
    /// Frame number.
    pub frame:       u64,
    /// Frame data (raw pixels).
    pub data:        Vec<u8>,
    /// Frame resolution.
    pub resolution:  Resolution,
    /// Last access timestamp.
    pub last_access: u64,
}

impl FrameCache {
    /// Creates a new frame cache.
    #[must_use]
    pub fn new(max_size_mb: usize) -> Self {
        Self {
            max_size:     max_size_mb * 1024 * 1024,
            current_size: 0,
            entries:      Vec::new(),
            hits:         0,
            misses:       0,
        }
    }

    /// Tries to get a frame from cache.
    pub fn get(&mut self, frame: u64) -> Option<&CachedFrame> {
        if let Some(pos) = self.entries.iter().position(|e| e.frame == frame) {
            self.hits += 1;
            // Update access time
            self.entries[pos].last_access = self.hits + self.misses;
            Some(&self.entries[pos])
        } else {
            self.misses += 1;
            None
        }
    }

    /// Puts a frame in the cache.
    pub fn put(&mut self, frame: u64, data: Vec<u8>, resolution: Resolution) {
        let frame_size = data.len();

        // Evict old frames if necessary
        while self.current_size + frame_size > self.max_size && !self.entries.is_empty() {
            self.evict_oldest();
        }

        // Don't cache if frame is larger than max size
        if frame_size > self.max_size {
            return;
        }

        // Remove existing entry for same frame
        if let Some(pos) = self.entries.iter().position(|e| e.frame == frame) {
            self.current_size -= self.entries[pos].data.len();
            self.entries.remove(pos);
        }

        self.entries.push(CachedFrame {
            frame,
            data,
            resolution,
            last_access: self.hits + self.misses,
        });
        self.current_size += frame_size;
    }

    /// Evicts the oldest entry.
    fn evict_oldest(&mut self) {
        if let Some((pos, _)) = self.entries.iter().enumerate().min_by_key(|(_, e)| e.last_access) {
            self.current_size -= self.entries[pos].data.len();
            self.entries.remove(pos);
        }
    }

    /// Clears the entire cache.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size = 0;
    }

    /// Returns the cache hit ratio.
    #[must_use]
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Returns current cache size in bytes.
    #[must_use]
    pub const fn size(&self) -> usize {
        self.current_size
    }

    /// Returns cache utilization percentage.
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            self.current_size as f64 / self.max_size as f64
        }
    }
}

/// Performance statistics for preview.
#[derive(Debug, Clone, Default)]
pub struct PreviewStats {
    /// Current FPS.
    pub fps:             f64,
    /// Average frame render time in ms.
    pub avg_render_time: f64,
    /// Dropped frames count.
    pub dropped_frames:  u64,
    /// Total frames rendered.
    pub total_frames:    u64,
    /// GPU memory used (bytes).
    pub gpu_memory_used: u64,
    /// CPU usage percentage.
    pub cpu_usage:       f64,
    /// Whether preview is real-time.
    pub is_realtime:     bool,
}

impl PreviewStats {
    /// Updates stats with new frame timing.
    pub fn update(&mut self, render_time_ms: f64, is_dropped: bool) {
        self.total_frames += 1;
        if is_dropped {
            self.dropped_frames += 1;
        }

        // Exponential moving average for render time
        self.avg_render_time = self.avg_render_time * 0.9 + render_time_ms * 0.1;

        // Update FPS
        if self.avg_render_time > 0.0 {
            self.fps = 1000.0 / self.avg_render_time;
        }
    }

    /// Returns the dropped frame percentage.
    #[must_use]
    pub fn drop_percentage(&self) -> f64 {
        if self.total_frames == 0 {
            0.0
        } else {
            self.dropped_frames as f64 / self.total_frames as f64 * 100.0
        }
    }
}

/// Audio monitoring settings.
#[derive(Debug, Clone)]
pub struct AudioMonitor {
    /// Whether audio is muted.
    pub muted:       bool,
    /// Volume level (0.0 to 1.0).
    pub volume:      f32,
    /// Solo track ID (if any).
    pub solo_track:  Option<u64>,
    /// Whether to scrub audio.
    pub scrub_audio: bool,
}

impl Default for AudioMonitor {
    fn default() -> Self {
        Self { muted: false, volume: 1.0, solo_track: None, scrub_audio: true }
    }
}

/// The main preview manager.
pub struct PreviewManager {
    /// Current playback state.
    state:              PlaybackState,
    /// Playback speed.
    speed:              PlaybackSpeed,
    /// Current position.
    position:           TimePosition,
    /// Timeline duration.
    duration:           TimePosition,
    /// Frame rate.
    frame_rate:         FrameRate,
    /// Preview quality.
    quality:            PreviewQuality,
    /// Loop mode.
    loop_mode:          LoopMode,
    /// In/out points.
    in_out:             InOutPoints,
    /// Frame cache.
    cache:              FrameCache,
    /// Performance stats.
    stats:              PreviewStats,
    /// Audio monitor.
    audio:              AudioMonitor,
    /// Source resolution.
    source_resolution:  Resolution,
    /// Preview resolution.
    preview_resolution: Resolution,
}

impl PreviewManager {
    /// Creates a new preview manager.
    #[must_use]
    pub fn new(duration: TimePosition, frame_rate: FrameRate, resolution: Resolution) -> Self {
        let quality = PreviewQuality::default();
        let preview_res = quality.calculate_resolution(resolution);

        Self {
            state: PlaybackState::Stopped,
            speed: PlaybackSpeed::default(),
            position: TimePosition::default(),
            duration,
            frame_rate,
            quality,
            loop_mode: LoopMode::default(),
            in_out: InOutPoints::new(),
            cache: FrameCache::new(512), // 512MB cache
            stats: PreviewStats::default(),
            audio: AudioMonitor::default(),
            source_resolution: resolution,
            preview_resolution: preview_res,
        }
    }

    /// Returns the current playback state.
    #[must_use]
    pub const fn state(&self) -> PlaybackState {
        self.state
    }

    /// Returns whether currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        matches!(
            self.state,
            PlaybackState::Playing | PlaybackState::ShuttleForward | PlaybackState::ShuttleBackward
        )
    }

    /// Returns the current position.
    #[must_use]
    pub const fn position(&self) -> TimePosition {
        self.position
    }

    /// Returns the current frame number.
    #[must_use]
    pub fn current_frame(&self) -> u64 {
        self.position_to_frame(self.position)
    }

    /// Converts position to frame number.
    #[must_use]
    pub fn position_to_frame(&self, position: TimePosition) -> u64 {
        let fps = self.frame_rate.as_f64();
        ((position.ms as f64 / 1000.0) * fps) as u64
    }

    /// Converts frame number to position.
    #[must_use]
    pub fn frame_to_position(&self, frame: u64) -> TimePosition {
        let fps = self.frame_rate.as_f64();
        let ms = ((frame as f64 / fps) * 1000.0) as u64;
        TimePosition::from_ms(ms)
    }

    /// Returns the total frame count.
    #[must_use]
    pub fn total_frames(&self) -> u64 {
        self.position_to_frame(self.duration)
    }

    /// Returns the timeline duration.
    #[must_use]
    pub const fn duration(&self) -> TimePosition {
        self.duration
    }

    /// Sets the timeline duration.
    pub fn set_duration(&mut self, duration: TimePosition) {
        self.duration = duration;
        if self.position.ms > duration.ms {
            self.position = duration;
        }
    }

    /// Starts playback.
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
        self.speed = PlaybackSpeed::NORMAL;
    }

    /// Pauses playback.
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stops playback and returns to start.
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.position = self.in_out.in_point.unwrap_or_default();
    }

    /// Toggles play/pause.
    pub fn toggle_playback(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            _ => self.play(),
        }
    }

    /// Seeks to a specific position.
    pub fn seek(&mut self, position: TimePosition) {
        self.position = TimePosition::from_ms(position.ms.min(self.duration.ms));
    }

    /// Seeks to a specific frame.
    pub fn seek_frame(&mut self, frame: u64) {
        self.seek(self.frame_to_position(frame));
    }

    /// Steps one frame forward.
    pub fn step_forward(&mut self) {
        self.state = PlaybackState::Stepping;
        let next_frame = self.current_frame() + 1;
        if next_frame <= self.total_frames() {
            self.seek_frame(next_frame);
        }
    }

    /// Steps one frame backward.
    pub fn step_backward(&mut self) {
        self.state = PlaybackState::Stepping;
        let current = self.current_frame();
        if current > 0 {
            self.seek_frame(current - 1);
        }
    }

    /// Goes to start.
    pub fn go_to_start(&mut self) {
        self.seek(self.in_out.in_point.unwrap_or_default());
    }

    /// Goes to end.
    pub fn go_to_end(&mut self) {
        self.seek(self.in_out.out_point.unwrap_or(self.duration));
    }

    /// Starts scrubbing.
    pub fn start_scrub(&mut self) {
        self.state = PlaybackState::Scrubbing;
    }

    /// Ends scrubbing.
    pub fn end_scrub(&mut self) {
        if self.state == PlaybackState::Scrubbing {
            self.state = PlaybackState::Paused;
        }
    }

    /// Updates scrub position.
    pub fn scrub_to(&mut self, position: TimePosition) {
        if self.state == PlaybackState::Scrubbing {
            self.seek(position);
        }
    }

    /// Sets playback speed.
    pub fn set_speed(&mut self, speed: PlaybackSpeed) {
        self.speed = speed;
        if speed.is_reverse() {
            self.state = PlaybackState::ShuttleBackward;
        } else if speed.abs() > 1.0 {
            self.state = PlaybackState::ShuttleForward;
        }
    }

    /// Returns current playback speed.
    #[must_use]
    pub const fn speed(&self) -> PlaybackSpeed {
        self.speed
    }

    /// Sets preview quality.
    pub fn set_quality(&mut self, quality: PreviewQuality) {
        self.quality = quality;
        self.preview_resolution = quality.calculate_resolution(self.source_resolution);
        self.cache.clear(); // Clear cache when quality changes
    }

    /// Returns preview quality.
    #[must_use]
    pub const fn quality(&self) -> PreviewQuality {
        self.quality
    }

    /// Returns preview resolution.
    #[must_use]
    pub const fn preview_resolution(&self) -> Resolution {
        self.preview_resolution
    }

    /// Sets loop mode.
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    /// Returns loop mode.
    #[must_use]
    pub const fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// Returns in/out points.
    #[must_use]
    pub fn in_out(&self) -> &InOutPoints {
        &self.in_out
    }

    /// Returns mutable in/out points.
    pub fn in_out_mut(&mut self) -> &mut InOutPoints {
        &mut self.in_out
    }

    /// Sets in point at current position.
    pub fn mark_in(&mut self) {
        self.in_out.set_in(self.position);
        self.in_out.active = true;
    }

    /// Sets out point at current position.
    pub fn mark_out(&mut self) {
        self.in_out.set_out(self.position);
        self.in_out.active = true;
    }

    /// Returns the frame cache.
    #[must_use]
    pub fn cache(&self) -> &FrameCache {
        &self.cache
    }

    /// Returns mutable frame cache.
    pub fn cache_mut(&mut self) -> &mut FrameCache {
        &mut self.cache
    }

    /// Returns performance stats.
    #[must_use]
    pub fn stats(&self) -> &PreviewStats {
        &self.stats
    }

    /// Returns audio monitor.
    #[must_use]
    pub fn audio(&self) -> &AudioMonitor {
        &self.audio
    }

    /// Returns mutable audio monitor.
    pub fn audio_mut(&mut self) -> &mut AudioMonitor {
        &mut self.audio
    }

    /// Updates playback position (called each frame).
    pub fn update(&mut self, delta_ms: f64) -> VideoEditorResult<()> {
        if !self.is_playing() {
            return Ok(());
        }

        let delta = (delta_ms * self.speed.value()) as i64;
        let new_pos = self.position.ms as i64 + delta;

        // Handle looping
        let (final_pos, should_loop) = self.calculate_loop_position(new_pos);
        self.position = TimePosition::from_ms(final_pos);

        if should_loop && self.loop_mode == LoopMode::PingPong {
            self.speed = PlaybackSpeed::new(-self.speed.value());
        }

        Ok(())
    }

    /// Calculates position considering loop mode.
    fn calculate_loop_position(&self, new_pos: i64) -> (u64, bool) {
        let max_pos = self.in_out.out_point.unwrap_or(self.duration).ms as i64;
        let min_pos = self.in_out.in_point.unwrap_or_default().ms as i64;

        match self.loop_mode {
            LoopMode::None => {
                if new_pos >= max_pos {
                    (max_pos as u64, false)
                } else if new_pos <= min_pos {
                    (min_pos as u64, false)
                } else {
                    (new_pos as u64, false)
                }
            },
            LoopMode::All | LoopMode::InOut => {
                if new_pos >= max_pos {
                    (min_pos as u64, true)
                } else if new_pos <= min_pos {
                    (max_pos as u64, true)
                } else {
                    (new_pos as u64, false)
                }
            },
            LoopMode::PingPong => {
                if new_pos >= max_pos || new_pos <= min_pos {
                    let clamped = (new_pos.max(min_pos).min(max_pos)) as u64;
                    (clamped, true)
                } else {
                    (new_pos as u64, false)
                }
            },
        }
    }
}

impl Default for PreviewManager {
    fn default() -> Self {
        Self::new(
            TimePosition::from_ms(60000), // 1 minute
            FrameRate::new(30, 1),
            Resolution::new(1920, 1080),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_manager_creation() {
        let manager = PreviewManager::default();
        assert_eq!(manager.state(), PlaybackState::Stopped);
        assert_eq!(manager.position().ms, 0);
    }

    #[test]
    fn test_playback_control() {
        let mut manager = PreviewManager::default();

        manager.play();
        assert!(manager.is_playing());

        manager.pause();
        assert_eq!(manager.state(), PlaybackState::Paused);

        manager.stop();
        assert_eq!(manager.state(), PlaybackState::Stopped);
    }

    #[test]
    fn test_seeking() {
        let mut manager = PreviewManager::default();

        manager.seek(TimePosition::from_ms(5000));
        assert_eq!(manager.position().ms, 5000);

        // Seek beyond duration should clamp
        manager.seek(TimePosition::from_ms(100000));
        assert_eq!(manager.position().ms, 60000);
    }

    #[test]
    fn test_frame_cache() {
        let mut cache = FrameCache::new(10); // 10MB

        cache.put(0, vec![0u8; 1024], Resolution::new(100, 100));
        assert!(cache.get(0).is_some());
        assert!(cache.get(1).is_none());
    }

    #[test]
    fn test_preview_quality() {
        let full = PreviewQuality::Full;
        let half = PreviewQuality::Half;

        let source = Resolution::new(1920, 1080);
        assert_eq!(full.calculate_resolution(source).width, 1920);
        assert_eq!(half.calculate_resolution(source).width, 960);
    }
}

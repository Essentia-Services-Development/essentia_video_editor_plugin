//! Audio Mixer for Essentia Video Editor Plugin
//! GAP-220-B-002: Audio Mixing System
//!
//! Features: Track mixing, volume control, pan, EQ, compression,
//! meters, ducking, and real-time audio monitoring.

use crate::errors::VideoEditorResult;

/// Unique identifier for an audio bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioBusId(u64);

impl AudioBusId {
    /// Creates a new audio bus ID.
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

/// Audio channel configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioChannelConfig {
    /// Mono (1 channel).
    Mono,
    /// Stereo (2 channels).
    #[default]
    Stereo,
    /// 5.1 surround (6 channels).
    Surround51,
    /// 7.1 surround (8 channels).
    Surround71,
    /// Custom channel count.
    Custom(u8),
}

impl AudioChannelConfig {
    /// Returns the number of channels.
    #[must_use]
    pub const fn channel_count(&self) -> u8 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
            Self::Custom(n) => *n,
        }
    }
}

/// Audio meter levels for visualization.
#[derive(Debug, Clone, Default)]
pub struct AudioMeterLevels {
    /// Peak level per channel (0.0 to 1.0+).
    pub peak:        Vec<f32>,
    /// RMS level per channel (0.0 to 1.0).
    pub rms:         Vec<f32>,
    /// Peak hold values.
    pub peak_hold:   Vec<f32>,
    /// Whether clipping occurred.
    pub is_clipping: bool,
}

impl AudioMeterLevels {
    /// Creates new meter levels for given channel count.
    #[must_use]
    pub fn new(channels: usize) -> Self {
        Self {
            peak:        vec![0.0; channels],
            rms:         vec![0.0; channels],
            peak_hold:   vec![0.0; channels],
            is_clipping: false,
        }
    }

    /// Updates peak levels with new sample data.
    pub fn update(&mut self, samples: &[f32], channels: usize) {
        if samples.is_empty() || channels == 0 {
            return;
        }

        let frames = samples.len() / channels;

        // Reset for new calculation
        for (i, (peak, rms)) in self.peak.iter_mut().zip(self.rms.iter_mut()).enumerate() {
            let mut max_sample: f32 = 0.0;
            let mut sum_squared: f64 = 0.0;

            for frame in 0..frames {
                let sample = samples[frame * channels + i].abs();
                max_sample = max_sample.max(sample);
                sum_squared += (sample * sample) as f64;
            }

            *peak = max_sample;
            *rms = (sum_squared / frames as f64).sqrt() as f32;

            // Update peak hold with decay
            if max_sample > self.peak_hold[i] {
                self.peak_hold[i] = max_sample;
            } else {
                self.peak_hold[i] *= 0.99; // Decay
            }

            if max_sample > 1.0 {
                self.is_clipping = true;
            }
        }
    }

    /// Resets all meter levels.
    pub fn reset(&mut self) {
        for peak in &mut self.peak {
            *peak = 0.0;
        }
        for rms in &mut self.rms {
            *rms = 0.0;
        }
        for hold in &mut self.peak_hold {
            *hold = 0.0;
        }
        self.is_clipping = false;
    }
}

/// Audio pan law determines how volume is distributed during panning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PanLaw {
    /// Linear pan (simple left/right balance).
    Linear,
    /// -3dB center (constant power).
    #[default]
    ConstantPower3dB,
    /// -4.5dB center.
    ConstantPower45dB,
    /// -6dB center (constant voltage).
    ConstantPower6dB,
}

impl PanLaw {
    /// Calculates left and right gain for a pan position (-1.0 to 1.0).
    #[must_use]
    pub fn calculate_gains(&self, pan: f32) -> (f32, f32) {
        let pan = pan.clamp(-1.0, 1.0);

        match self {
            Self::Linear => {
                let left = if pan < 0.0 { 1.0 } else { 1.0 - pan };
                let right = if pan > 0.0 { 1.0 } else { 1.0 + pan };
                (left, right)
            },
            Self::ConstantPower3dB => {
                let angle = (pan + 1.0) * core::f32::consts::FRAC_PI_4;
                (angle.cos(), angle.sin())
            },
            Self::ConstantPower45dB => {
                let angle = (pan + 1.0) * core::f32::consts::FRAC_PI_4;
                let left = angle.cos();
                let right = angle.sin();
                // Apply additional -1.5dB at center
                let center_cut: f32 = 0.841; // -1.5dB
                (
                    left * center_cut.powf(1.0 - pan.abs()),
                    right * center_cut.powf(1.0 - pan.abs()),
                )
            },
            Self::ConstantPower6dB => {
                let left = ((1.0 - pan) * 0.5).sqrt();
                let right = ((1.0 + pan) * 0.5).sqrt();
                (left, right)
            },
        }
    }
}

/// Audio track strip with volume, pan, and effects.
#[derive(Debug, Clone)]
pub struct AudioTrackStrip {
    /// Track ID this strip is associated with.
    track_id:   u64,
    /// Track name.
    name:       String,
    /// Volume level (0.0 to 2.0, where 1.0 = unity gain).
    volume:     f32,
    /// Pan position (-1.0 = left, 0.0 = center, 1.0 = right).
    pan:        f32,
    /// Whether the track is muted.
    muted:      bool,
    /// Whether the track is soloed.
    solo:       bool,
    /// Output bus ID.
    output_bus: AudioBusId,
    /// Channel configuration.
    channels:   AudioChannelConfig,
    /// Current meter levels.
    meters:     AudioMeterLevels,
    /// Insert effects.
    inserts:    Vec<AudioInsert>,
    /// Send levels to aux buses.
    sends:      Vec<AudioSend>,
}

/// An audio insert effect.
#[derive(Debug, Clone)]
pub struct AudioInsert {
    /// Insert slot index.
    pub slot:       u8,
    /// Effect type.
    pub effect:     AudioEffectType,
    /// Effect parameters.
    pub parameters: AudioEffectParams,
    /// Whether insert is bypassed.
    pub bypassed:   bool,
}

/// Audio effect types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioEffectType {
    /// Parametric EQ.
    ParametricEQ,
    /// Compressor.
    Compressor,
    /// Limiter.
    Limiter,
    /// Noise gate.
    NoiseGate,
    /// De-esser.
    DeEsser,
    /// Reverb.
    Reverb,
    /// Delay.
    Delay,
    /// Chorus.
    Chorus,
    /// Low-pass filter.
    LowPassFilter,
    /// High-pass filter.
    HighPassFilter,
    /// Notch filter.
    NotchFilter,
}

/// Parameters for audio effects.
#[derive(Debug, Clone, Default)]
pub struct AudioEffectParams {
    /// Generic parameter map.
    pub params: Vec<(String, f32)>,
}

impl AudioEffectParams {
    /// Creates new empty parameters.
    #[must_use]
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    /// Sets a parameter value.
    pub fn set(&mut self, name: impl Into<String>, value: f32) {
        let name = name.into();
        if let Some(param) = self.params.iter_mut().find(|(n, _)| n == &name) {
            param.1 = value;
        } else {
            self.params.push((name, value));
        }
    }

    /// Gets a parameter value.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<f32> {
        self.params.iter().find(|(n, _)| n == name).map(|(_, v)| *v)
    }

    /// Creates default EQ parameters.
    #[must_use]
    pub fn default_eq() -> Self {
        let mut params = Self::new();
        params.set("low_gain", 0.0);
        params.set("low_freq", 100.0);
        params.set("mid_gain", 0.0);
        params.set("mid_freq", 1000.0);
        params.set("mid_q", 1.0);
        params.set("high_gain", 0.0);
        params.set("high_freq", 8000.0);
        params
    }

    /// Creates default compressor parameters.
    #[must_use]
    pub fn default_compressor() -> Self {
        let mut params = Self::new();
        params.set("threshold", -20.0);
        params.set("ratio", 4.0);
        params.set("attack", 10.0);
        params.set("release", 100.0);
        params.set("makeup_gain", 0.0);
        params.set("knee", 6.0);
        params
    }
}

/// Audio send to an auxiliary bus.
#[derive(Debug, Clone)]
pub struct AudioSend {
    /// Target bus ID.
    pub bus_id:    AudioBusId,
    /// Send level (0.0 to 1.0).
    pub level:     f32,
    /// Whether send is pre-fader.
    pub pre_fader: bool,
    /// Whether send is muted.
    pub muted:     bool,
}

impl AudioTrackStrip {
    /// Creates a new audio track strip.
    #[must_use]
    pub fn new(track_id: u64, name: impl Into<String>, output_bus: AudioBusId) -> Self {
        Self {
            track_id,
            name: name.into(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            output_bus,
            channels: AudioChannelConfig::Stereo,
            meters: AudioMeterLevels::new(2),
            inserts: Vec::new(),
            sends: Vec::new(),
        }
    }

    /// Returns the track ID.
    #[must_use]
    pub const fn track_id(&self) -> u64 {
        self.track_id
    }

    /// Returns the track name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the track name.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Returns the volume level.
    #[must_use]
    pub const fn volume(&self) -> f32 {
        self.volume
    }

    /// Sets the volume level.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }

    /// Returns the pan position.
    #[must_use]
    pub const fn pan(&self) -> f32 {
        self.pan
    }

    /// Sets the pan position.
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Returns whether the track is muted.
    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    /// Sets the muted state.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Toggles the muted state.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    /// Returns whether the track is soloed.
    #[must_use]
    pub const fn is_solo(&self) -> bool {
        self.solo
    }

    /// Sets the solo state.
    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
    }

    /// Toggles the solo state.
    pub fn toggle_solo(&mut self) {
        self.solo = !self.solo;
    }

    /// Returns the output bus.
    #[must_use]
    pub const fn output_bus(&self) -> AudioBusId {
        self.output_bus
    }

    /// Sets the output bus.
    pub fn set_output_bus(&mut self, bus: AudioBusId) {
        self.output_bus = bus;
    }

    /// Returns the current meter levels.
    #[must_use]
    pub fn meters(&self) -> &AudioMeterLevels {
        &self.meters
    }

    /// Updates meter levels with new audio data.
    pub fn update_meters(&mut self, samples: &[f32]) {
        self.meters.update(samples, self.channels.channel_count() as usize);
    }

    /// Adds an insert effect.
    pub fn add_insert(&mut self, effect: AudioEffectType) -> u8 {
        let slot = self.inserts.len() as u8;
        self.inserts.push(AudioInsert {
            slot,
            effect,
            parameters: AudioEffectParams::new(),
            bypassed: false,
        });
        slot
    }

    /// Removes an insert effect by slot.
    pub fn remove_insert(&mut self, slot: u8) -> bool {
        if let Some(pos) = self.inserts.iter().position(|i| i.slot == slot) {
            self.inserts.remove(pos);
            // Reindex slots
            for (i, insert) in self.inserts.iter_mut().enumerate() {
                insert.slot = i as u8;
            }
            true
        } else {
            false
        }
    }

    /// Returns all insert effects.
    #[must_use]
    pub fn inserts(&self) -> &[AudioInsert] {
        &self.inserts
    }

    /// Adds a send to an auxiliary bus.
    pub fn add_send(&mut self, bus_id: AudioBusId, level: f32, pre_fader: bool) {
        self.sends.push(AudioSend {
            bus_id,
            level: level.clamp(0.0, 1.0),
            pre_fader,
            muted: false,
        });
    }

    /// Returns all sends.
    #[must_use]
    pub fn sends(&self) -> &[AudioSend] {
        &self.sends
    }

    /// Calculates the effective gain for this track.
    #[must_use]
    pub fn effective_gain(&self, pan_law: PanLaw) -> (f32, f32) {
        if self.muted {
            return (0.0, 0.0);
        }

        let (left_pan, right_pan) = pan_law.calculate_gains(self.pan);
        (self.volume * left_pan, self.volume * right_pan)
    }
}

/// Audio bus for mixing multiple tracks.
#[derive(Debug, Clone)]
pub struct AudioBus {
    /// Bus identifier.
    id:       AudioBusId,
    /// Bus name.
    name:     String,
    /// Bus type.
    bus_type: AudioBusType,
    /// Volume level.
    volume:   f32,
    /// Pan position.
    pan:      f32,
    /// Whether bus is muted.
    muted:    bool,
    /// Whether bus is soloed.
    solo:     bool,
    /// Output bus (None for master).
    output:   Option<AudioBusId>,
    /// Current meter levels.
    meters:   AudioMeterLevels,
    /// Insert effects.
    inserts:  Vec<AudioInsert>,
}

/// Type of audio bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AudioBusType {
    /// Master output bus.
    #[default]
    Master,
    /// Auxiliary/effects bus.
    Aux,
    /// Group/submix bus.
    Group,
}

impl AudioBus {
    /// Creates a new audio bus.
    #[must_use]
    pub fn new(id: AudioBusId, name: impl Into<String>, bus_type: AudioBusType) -> Self {
        Self {
            id,
            name: name.into(),
            bus_type,
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            output: None,
            meters: AudioMeterLevels::new(2),
            inserts: Vec::new(),
        }
    }

    /// Creates the master bus.
    #[must_use]
    pub fn master() -> Self {
        Self::new(AudioBusId::new(0), "Master", AudioBusType::Master)
    }

    /// Returns the bus ID.
    #[must_use]
    pub const fn id(&self) -> AudioBusId {
        self.id
    }

    /// Returns the bus name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the bus type.
    #[must_use]
    pub const fn bus_type(&self) -> AudioBusType {
        self.bus_type
    }

    /// Returns the volume level.
    #[must_use]
    pub const fn volume(&self) -> f32 {
        self.volume
    }

    /// Sets the volume level.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 2.0);
    }

    /// Returns whether the bus is muted.
    #[must_use]
    pub const fn is_muted(&self) -> bool {
        self.muted
    }

    /// Returns the current meter levels.
    #[must_use]
    pub fn meters(&self) -> &AudioMeterLevels {
        &self.meters
    }
}

/// The main audio mixer.
pub struct AudioMixer {
    /// Master output bus.
    master:      AudioBus,
    /// Auxiliary buses.
    aux_buses:   Vec<AudioBus>,
    /// Group buses.
    group_buses: Vec<AudioBus>,
    /// Track strips.
    tracks:      Vec<AudioTrackStrip>,
    /// Pan law setting.
    pan_law:     PanLaw,
    /// Sample rate.
    sample_rate: u32,
    /// Buffer size.
    buffer_size: usize,
    /// Next bus ID counter.
    next_bus_id: u64,
    /// Whether any track is soloed.
    has_solo:    bool,
}

impl AudioMixer {
    /// Creates a new audio mixer.
    #[must_use]
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            master: AudioBus::master(),
            aux_buses: Vec::new(),
            group_buses: Vec::new(),
            tracks: Vec::new(),
            pan_law: PanLaw::default(),
            sample_rate,
            buffer_size,
            next_bus_id: 1, // 0 is reserved for master
            has_solo: false,
        }
    }

    /// Returns the master bus.
    #[must_use]
    pub fn master(&self) -> &AudioBus {
        &self.master
    }

    /// Returns mutable master bus.
    pub fn master_mut(&mut self) -> &mut AudioBus {
        &mut self.master
    }

    /// Adds a new track strip.
    ///
    /// # Errors
    ///
    /// Returns `VideoEditorError::Timeline` if the track cannot be created.
    pub fn add_track(
        &mut self, track_id: u64, name: impl Into<String>,
    ) -> crate::errors::VideoEditorResult<&mut AudioTrackStrip> {
        self.tracks.push(AudioTrackStrip::new(track_id, name, self.master.id()));
        // SAFETY: Element was just pushed, so last_mut will always succeed
        self.tracks.last_mut().ok_or_else(|| {
            crate::VideoEditorError::Timeline("Track was just added but not found".to_string())
        })
    }

    /// Removes a track strip.
    pub fn remove_track(&mut self, track_id: u64) -> bool {
        if let Some(pos) = self.tracks.iter().position(|t| t.track_id() == track_id) {
            self.tracks.remove(pos);
            self.update_solo_state();
            true
        } else {
            false
        }
    }

    /// Gets a track strip by track ID.
    #[must_use]
    pub fn get_track(&self, track_id: u64) -> Option<&AudioTrackStrip> {
        self.tracks.iter().find(|t| t.track_id() == track_id)
    }

    /// Gets a mutable track strip by track ID.
    pub fn get_track_mut(&mut self, track_id: u64) -> Option<&mut AudioTrackStrip> {
        self.tracks.iter_mut().find(|t| t.track_id() == track_id)
    }

    /// Returns all track strips.
    #[must_use]
    pub fn tracks(&self) -> &[AudioTrackStrip] {
        &self.tracks
    }

    /// Creates a new auxiliary bus.
    pub fn create_aux_bus(&mut self, name: impl Into<String>) -> AudioBusId {
        let id = AudioBusId::new(self.next_bus_id);
        self.next_bus_id += 1;

        let mut bus = AudioBus::new(id, name, AudioBusType::Aux);
        bus.output = Some(self.master.id());

        self.aux_buses.push(bus);
        id
    }

    /// Creates a new group bus.
    pub fn create_group_bus(&mut self, name: impl Into<String>) -> AudioBusId {
        let id = AudioBusId::new(self.next_bus_id);
        self.next_bus_id += 1;

        let mut bus = AudioBus::new(id, name, AudioBusType::Group);
        bus.output = Some(self.master.id());

        self.group_buses.push(bus);
        id
    }

    /// Gets a bus by ID.
    #[must_use]
    pub fn get_bus(&self, id: AudioBusId) -> Option<&AudioBus> {
        if id.inner() == 0 {
            return Some(&self.master);
        }
        self.aux_buses
            .iter()
            .find(|b| b.id() == id)
            .or_else(|| self.group_buses.iter().find(|b| b.id() == id))
    }

    /// Returns the pan law setting.
    #[must_use]
    pub const fn pan_law(&self) -> PanLaw {
        self.pan_law
    }

    /// Sets the pan law.
    pub fn set_pan_law(&mut self, pan_law: PanLaw) {
        self.pan_law = pan_law;
    }

    /// Returns the sample rate.
    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Updates the solo state based on track settings.
    fn update_solo_state(&mut self) {
        self.has_solo = self.tracks.iter().any(|t| t.is_solo());
    }

    /// Sets solo state for a track.
    pub fn set_track_solo(&mut self, track_id: u64, solo: bool) {
        if let Some(track) = self.get_track_mut(track_id) {
            track.set_solo(solo);
        }
        self.update_solo_state();
    }

    /// Returns whether the mixer has any soloed tracks.
    #[must_use]
    pub const fn has_solo(&self) -> bool {
        self.has_solo
    }

    /// Checks if a track should be audible (considering solo state).
    #[must_use]
    pub fn is_track_audible(&self, track_id: u64) -> bool {
        let Some(track) = self.get_track(track_id) else {
            return false;
        };

        if track.is_muted() {
            return false;
        }

        if self.has_solo { track.is_solo() } else { true }
    }

    /// Processes audio through the mixer (stub for GPU/DSP implementation).
    pub fn process(&mut self, _input: &[f32], _output: &mut [f32]) -> VideoEditorResult<()> {
        // In a full implementation, this would:
        // 1. Route track audio through inserts
        // 2. Apply volume and pan
        // 3. Sum into buses
        // 4. Apply bus processing
        // 5. Mix to master
        // 6. Update all meters
        Ok(())
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new(48000, 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_mixer_creation() {
        let mixer = AudioMixer::new(48000, 1024);
        assert_eq!(mixer.sample_rate(), 48000);
        assert!(matches!(mixer.pan_law(), PanLaw::ConstantPower3dB));
    }

    #[test]
    fn test_track_strip() {
        let mut mixer = AudioMixer::new(48000, 1024);
        let track = mixer.add_track(1, "Audio 1").unwrap();

        assert_eq!(track.track_id(), 1);
        assert_eq!(track.name(), "Audio 1");
        assert!((track.volume() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pan_law() {
        let law = PanLaw::ConstantPower3dB;
        let (left, right) = law.calculate_gains(0.0);

        // At center, both should be roughly equal (~0.707)
        assert!((left - right).abs() < 0.01);
        assert!(left > 0.7 && left < 0.72);
    }

    #[test]
    fn test_solo_logic() {
        let mut mixer = AudioMixer::new(48000, 1024);
        let _ = mixer.add_track(1, "Track 1");
        let _ = mixer.add_track(2, "Track 2");

        assert!(mixer.is_track_audible(1));
        assert!(mixer.is_track_audible(2));

        mixer.set_track_solo(1, true);
        assert!(mixer.is_track_audible(1));
        assert!(!mixer.is_track_audible(2));
    }

    #[test]
    fn test_meter_levels() {
        let mut meters = AudioMeterLevels::new(2);
        let samples = vec![0.5, -0.3, 0.8, -0.2, 0.1, 0.6];
        meters.update(&samples, 2);

        assert!(meters.peak[0] > 0.0);
        assert!(meters.peak[1] > 0.0);
        assert!(!meters.is_clipping);
    }
}

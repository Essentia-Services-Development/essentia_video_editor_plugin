//! Effects pipeline.

/// Video effect.
#[derive(Debug, Clone)]
pub struct VideoEffect {
    /// Effect identifier.
    pub id:          u64,
    /// Effect type.
    pub effect_type: EffectType,
    /// Effect parameters.
    pub parameters:  Vec<(String, f64)>,
}

/// Effect type.
#[derive(Debug, Clone, Copy)]
pub enum EffectType {
    /// Color correction.
    ColorCorrection,
    /// Blur effect.
    Blur,
    /// Sharpen effect.
    Sharpen,
    /// Fade in/out.
    Fade,
    /// Cross dissolve.
    CrossDissolve,
    /// Custom shader.
    CustomShader,
}

/// Effects pipeline for video processing.
pub struct EffectsPipeline {
    effects:        Vec<VideoEffect>,
    next_effect_id: u64,
}

impl EffectsPipeline {
    /// Create a new effects pipeline.
    pub fn new() -> Self {
        Self { effects: Vec::new(), next_effect_id: 1 }
    }

    /// Add an effect.
    pub fn add_effect(&mut self, effect_type: EffectType) -> u64 {
        let id = self.next_effect_id;
        self.next_effect_id += 1;

        self.effects.push(VideoEffect { id, effect_type, parameters: Vec::new() });

        id
    }

    /// Remove an effect.
    pub fn remove_effect(&mut self, effect_id: u64) -> bool {
        if let Some(pos) = self.effects.iter().position(|e| e.id == effect_id) {
            self.effects.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all effects.
    pub fn effects(&self) -> &[VideoEffect] {
        &self.effects
    }
}

impl Default for EffectsPipeline {
    fn default() -> Self {
        Self::new()
    }
}

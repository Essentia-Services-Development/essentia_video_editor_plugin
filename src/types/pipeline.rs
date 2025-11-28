//! Render pipeline configuration types.
//!
//! GPU-accelerated pipeline configuration for video processing.

use std::collections::HashMap;

/// GPU backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GpuBackend {
    /// Automatic selection.
    #[default]
    Auto,
    /// Vulkan.
    Vulkan,
    /// DirectX 12.
    Dx12,
    /// Metal (macOS/iOS).
    Metal,
    /// OpenGL.
    OpenGl,
    /// WebGPU.
    WebGpu,
    /// CPU fallback (no GPU).
    Cpu,
}

impl GpuBackend {
    /// Returns the backend name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Vulkan => "Vulkan",
            Self::Dx12 => "DirectX 12",
            Self::Metal => "Metal",
            Self::OpenGl => "OpenGL",
            Self::WebGpu => "WebGPU",
            Self::Cpu => "CPU",
        }
    }

    /// Returns whether this is a GPU backend.
    #[must_use]
    pub const fn is_gpu(&self) -> bool {
        !matches!(self, Self::Cpu)
    }
}

/// Render quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QualityPreset {
    /// Draft quality - fastest.
    Draft,
    /// Preview quality.
    Preview,
    /// Standard quality.
    #[default]
    Standard,
    /// High quality.
    High,
    /// Ultra quality - highest.
    Ultra,
    /// Custom settings.
    Custom,
}

impl QualityPreset {
    /// Returns the preset name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::Preview => "Preview",
            Self::Standard => "Standard",
            Self::High => "High",
            Self::Ultra => "Ultra",
            Self::Custom => "Custom",
        }
    }

    /// Returns the recommended sample count for this preset.
    #[must_use]
    pub const fn sample_count(&self) -> u32 {
        match self {
            Self::Draft => 1,
            Self::Preview => 1,
            Self::Standard => 4,
            Self::High => 8,
            Self::Ultra => 16,
            Self::Custom => 4,
        }
    }

    /// Returns the recommended motion blur samples.
    #[must_use]
    pub const fn motion_blur_samples(&self) -> u32 {
        match self {
            Self::Draft => 0,
            Self::Preview => 4,
            Self::Standard => 16,
            Self::High => 32,
            Self::Ultra => 64,
            Self::Custom => 16,
        }
    }
}

/// Render pipeline configuration.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// GPU backend to use.
    pub backend:             GpuBackend,
    /// Quality preset.
    pub quality:             QualityPreset,
    /// Maximum texture resolution.
    pub max_texture_size:    u32,
    /// Enable motion blur.
    pub motion_blur_enabled: bool,
    /// Motion blur samples.
    pub motion_blur_samples: u32,
    /// Enable depth of field.
    pub dof_enabled:         bool,
    /// Anti-aliasing mode.
    pub antialiasing:        AntialiasingMode,
    /// Enable HDR rendering.
    pub hdr_enabled:         bool,
    /// Enable color management.
    pub color_managed:       bool,
    /// Cache size in MB.
    pub cache_size_mb:       u32,
    /// Enable GPU memory pooling.
    pub memory_pooling:      bool,
    /// Parallel render threads (0 = auto).
    pub render_threads:      u32,
    /// Enable async compute.
    pub async_compute:       bool,
    /// Custom shader defines.
    pub shader_defines:      HashMap<String, String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            backend:             GpuBackend::Auto,
            quality:             QualityPreset::Standard,
            max_texture_size:    16384,
            motion_blur_enabled: true,
            motion_blur_samples: 16,
            dof_enabled:         true,
            antialiasing:        AntialiasingMode::Msaa4x,
            hdr_enabled:         true,
            color_managed:       true,
            cache_size_mb:       2048,
            memory_pooling:      true,
            render_threads:      0,
            async_compute:       true,
            shader_defines:      HashMap::new(),
        }
    }
}

impl PipelineConfig {
    /// Creates a draft quality config for fast previews.
    #[must_use]
    pub fn draft() -> Self {
        Self {
            quality: QualityPreset::Draft,
            motion_blur_enabled: false,
            motion_blur_samples: 0,
            dof_enabled: false,
            antialiasing: AntialiasingMode::None,
            cache_size_mb: 512,
            ..Default::default()
        }
    }

    /// Creates a high quality config for final renders.
    #[must_use]
    pub fn high_quality() -> Self {
        Self {
            quality: QualityPreset::High,
            max_texture_size: 32768,
            motion_blur_samples: 32,
            antialiasing: AntialiasingMode::Msaa8x,
            cache_size_mb: 4096,
            ..Default::default()
        }
    }

    /// Creates an ultra quality config.
    #[must_use]
    pub fn ultra() -> Self {
        Self {
            quality: QualityPreset::Ultra,
            max_texture_size: 65536,
            motion_blur_samples: 64,
            antialiasing: AntialiasingMode::Msaa16x,
            cache_size_mb: 8192,
            ..Default::default()
        }
    }

    /// Sets a shader define.
    pub fn set_define(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.shader_defines.insert(key.into(), value.into());
    }
}

/// Anti-aliasing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AntialiasingMode {
    /// No anti-aliasing.
    None,
    /// MSAA 2x.
    Msaa2x,
    /// MSAA 4x.
    #[default]
    Msaa4x,
    /// MSAA 8x.
    Msaa8x,
    /// MSAA 16x.
    Msaa16x,
    /// FXAA (fast approximate).
    Fxaa,
    /// TAA (temporal).
    Taa,
    /// SMAA (subpixel morphological).
    Smaa,
}

impl AntialiasingMode {
    /// Returns the sample count for MSAA modes.
    #[must_use]
    pub const fn sample_count(&self) -> u32 {
        match self {
            Self::None | Self::Fxaa | Self::Smaa => 1,
            Self::Msaa2x => 2,
            Self::Msaa4x | Self::Taa => 4,
            Self::Msaa8x => 8,
            Self::Msaa16x => 16,
        }
    }

    /// Returns whether this mode requires a resolve pass.
    #[must_use]
    pub const fn requires_resolve(&self) -> bool {
        matches!(
            self,
            Self::Msaa2x | Self::Msaa4x | Self::Msaa8x | Self::Msaa16x
        )
    }
}

/// Render pass type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderPassType {
    /// Geometry pass.
    Geometry,
    /// Shadow pass.
    Shadow,
    /// Lighting pass.
    Lighting,
    /// Post-processing pass.
    PostProcess,
    /// Composite pass.
    Composite,
    /// UI overlay pass.
    Ui,
    /// Final output pass.
    Output,
}

/// Render pass configuration.
#[derive(Debug, Clone)]
pub struct RenderPass {
    /// Pass type.
    pub pass_type:    RenderPassType,
    /// Pass name.
    pub name:         String,
    /// Whether pass is enabled.
    pub enabled:      bool,
    /// Input attachments.
    pub inputs:       Vec<String>,
    /// Output attachments.
    pub outputs:      Vec<String>,
    /// Clear color (RGBA).
    pub clear_color:  Option<[f32; 4]>,
    /// Clear depth.
    pub clear_depth:  Option<f32>,
    /// Pass dependencies.
    pub dependencies: Vec<String>,
}

impl RenderPass {
    /// Creates a new render pass.
    #[must_use]
    pub fn new(pass_type: RenderPassType, name: impl Into<String>) -> Self {
        Self {
            pass_type,
            name: name.into(),
            enabled: true,
            inputs: Vec::new(),
            outputs: Vec::new(),
            clear_color: None,
            clear_depth: None,
            dependencies: Vec::new(),
        }
    }

    /// Sets the clear color.
    #[must_use]
    pub fn with_clear_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.clear_color = Some([r, g, b, a]);
        self
    }

    /// Sets the clear depth.
    #[must_use]
    pub fn with_clear_depth(mut self, depth: f32) -> Self {
        self.clear_depth = Some(depth);
        self
    }

    /// Adds an input attachment.
    #[must_use]
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.inputs.push(input.into());
        self
    }

    /// Adds an output attachment.
    #[must_use]
    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.outputs.push(output.into());
        self
    }
}

/// Render graph for compositing pipeline.
#[derive(Debug, Clone, Default)]
pub struct RenderGraph {
    /// All render passes.
    passes:      Vec<RenderPass>,
    /// Pass execution order.
    order:       Vec<usize>,
    /// Named attachments.
    attachments: HashMap<String, AttachmentConfig>,
}

impl RenderGraph {
    /// Creates a new empty render graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a render pass.
    pub fn add_pass(&mut self, pass: RenderPass) {
        self.passes.push(pass);
        self.order.push(self.passes.len() - 1);
    }

    /// Defines an attachment.
    pub fn define_attachment(&mut self, name: impl Into<String>, config: AttachmentConfig) {
        self.attachments.insert(name.into(), config);
    }

    /// Returns the number of passes.
    #[must_use]
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Returns the pass execution order.
    #[must_use]
    pub fn execution_order(&self) -> &[usize] {
        &self.order
    }
}

/// Attachment configuration.
#[derive(Debug, Clone)]
pub struct AttachmentConfig {
    /// Width (0 = framebuffer width).
    pub width:    u32,
    /// Height (0 = framebuffer height).
    pub height:   u32,
    /// Pixel format.
    pub format:   AttachmentFormat,
    /// Sample count.
    pub samples:  u32,
    /// Load operation.
    pub load_op:  LoadOp,
    /// Store operation.
    pub store_op: StoreOp,
}

impl Default for AttachmentConfig {
    fn default() -> Self {
        Self {
            width:    0,
            height:   0,
            format:   AttachmentFormat::Rgba16Float,
            samples:  1,
            load_op:  LoadOp::Clear,
            store_op: StoreOp::Store,
        }
    }
}

/// Attachment format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AttachmentFormat {
    /// RGBA 8-bit unorm.
    Rgba8Unorm,
    /// RGBA 8-bit sRGB.
    Rgba8Srgb,
    /// RGBA 16-bit float.
    #[default]
    Rgba16Float,
    /// RGBA 32-bit float.
    Rgba32Float,
    /// Depth 16-bit.
    Depth16,
    /// Depth 24-bit.
    Depth24,
    /// Depth 32-bit float.
    Depth32Float,
    /// Depth 24 + Stencil 8.
    Depth24Stencil8,
}

/// Load operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LoadOp {
    /// Load existing content.
    Load,
    /// Clear to value.
    #[default]
    Clear,
    /// Don't care about initial content.
    DontCare,
}

/// Store operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StoreOp {
    /// Store content.
    #[default]
    Store,
    /// Don't care about content.
    DontCare,
}

/// Shader compilation options.
#[derive(Debug, Clone, Default)]
pub struct ShaderOptions {
    /// Shader entry point.
    pub entry_point: String,
    /// Shader stage.
    pub stage:       ShaderStage,
    /// Optimization level.
    pub optimize:    OptimizationLevel,
    /// Debug info enabled.
    pub debug_info:  bool,
    /// Defines.
    pub defines:     HashMap<String, String>,
}

/// Shader stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ShaderStage {
    /// Vertex shader.
    #[default]
    Vertex,
    /// Fragment/pixel shader.
    Fragment,
    /// Compute shader.
    Compute,
    /// Geometry shader.
    Geometry,
    /// Tessellation control.
    TessControl,
    /// Tessellation evaluation.
    TessEvaluation,
}

/// Optimization level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OptimizationLevel {
    /// No optimization.
    None,
    /// Basic optimization.
    Basic,
    /// Performance optimization.
    #[default]
    Performance,
    /// Size optimization.
    Size,
}

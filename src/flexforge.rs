//! FlexForge Integration for Essentia Video Editor Plugin
//!
//! Implements comprehensive FlexForge integration for the video editor plugin,
//! providing timeline editor, effects configuration, and GPU pipeline settings.
//!
//! ## Features
//!
//! - Non-linear timeline editor integration
//! - Effects and transitions configuration
//! - GPU rendering pipeline settings
//! - Real-time preview streaming (60fps)
//! - Asset library browser

use std::sync::{Arc, Mutex};

use essentia_traits::plugin_contracts::{
    ConfigField, ConfigSchema, EditorAction, EditorPresentable, FlexForgeCapability,
    FlexForgeIntegration, FlexForgePanelCategory, FlexForgePanelInfo, StreamingCapable,
    UiConfigurable,
};

// ============================================================================
// Configuration Types
// ============================================================================

/// Video editor configuration.
#[derive(Debug, Clone)]
pub struct VideoEditorConfig {
    // Rendering settings
    /// Default resolution width
    pub resolution_width:      u32,
    /// Default resolution height
    pub resolution_height:     u32,
    /// Frame rate
    pub frame_rate:            u32,
    /// Enable GPU acceleration
    pub gpu_acceleration:      bool,
    /// Preview quality (0-100)
    pub preview_quality:       u32,
    // Timeline settings
    /// Auto-save interval (seconds)
    pub autosave_interval_sec: u32,
    /// Undo history depth
    pub undo_depth:            u32,
    /// Snap to grid enabled
    pub snap_to_grid:          bool,
    // Export settings
    /// Default export codec
    pub export_codec:          String,
    /// Export bitrate (Mbps)
    pub export_bitrate_mbps:   u32,
    /// Enable hardware encoding
    pub hardware_encoding:     bool,
    // AI settings
    /// Enable AI scene detection
    pub ai_scene_detection:    bool,
    /// Enable AI color grading
    pub ai_color_grading:      bool,
}

impl Default for VideoEditorConfig {
    fn default() -> Self {
        Self {
            resolution_width:      1920,
            resolution_height:     1080,
            frame_rate:            60,
            gpu_acceleration:      true,
            preview_quality:       75,
            autosave_interval_sec: 300,
            undo_depth:            100,
            snap_to_grid:          true,
            export_codec:          String::from("h265"),
            export_bitrate_mbps:   50,
            hardware_encoding:     true,
            ai_scene_detection:    true,
            ai_color_grading:      false,
        }
    }
}

/// Video editor metrics.
#[derive(Debug, Clone, Default)]
pub struct VideoEditorMetrics {
    /// Current playback position (ms)
    pub playback_position_ms: u64,
    /// Total timeline duration (ms)
    pub timeline_duration_ms: u64,
    /// GPU memory usage (bytes)
    pub gpu_memory_bytes:     u64,
    /// Render FPS
    pub render_fps:           f32,
    /// Active tracks count
    pub active_tracks:        u32,
    /// Processing status
    pub processing:           bool,
}

// ============================================================================
// FlexForge Integration
// ============================================================================

/// FlexForge integration for Video Editor plugin.
#[derive(Debug)]
pub struct VideoEditorFlexForge {
    /// Current configuration
    config:           Arc<Mutex<VideoEditorConfig>>,
    /// Current metrics
    metrics:          Arc<Mutex<VideoEditorMetrics>>,
    /// Streaming active flag
    stream_active:    bool,
    /// Current stream ID
    stream_id:        Option<u64>,
    /// Next stream ID counter
    next_id:          u64,
    /// Current project path
    current_project:  Option<String>,
    /// Project modified flag
    project_modified: bool,
}

impl VideoEditorFlexForge {
    /// Creates a new FlexForge integration wrapper.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config:           Arc::new(Mutex::new(VideoEditorConfig::default())),
            metrics:          Arc::new(Mutex::new(VideoEditorMetrics::default())),
            stream_active:    false,
            stream_id:        None,
            next_id:          1,
            current_project:  None,
            project_modified: false,
        }
    }

    /// Returns panel info with capabilities.
    #[must_use]
    pub fn panel_info(&self) -> FlexForgePanelInfo {
        FlexForgePanelInfo {
            id:           self.panel_id().to_string(),
            name:         self.display_name().to_string(),
            category:     self.category(),
            icon:         self.icon_glyph().map(String::from),
            priority:     self.priority(),
            capabilities: vec![
                FlexForgeCapability::Configuration,
                FlexForgeCapability::Editor,
                FlexForgeCapability::Streaming,
                FlexForgeCapability::Visualization,
            ],
        }
    }

    /// Updates video editor metrics.
    pub fn update_metrics(&mut self, metrics: VideoEditorMetrics) {
        if let Ok(mut m) = self.metrics.lock() {
            *m = metrics;
        }
    }

    /// Returns current render FPS.
    #[must_use]
    pub fn render_fps(&self) -> f32 {
        self.metrics.lock().map(|m| m.render_fps).unwrap_or(0.0)
    }

    fn next_stream_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

impl Default for VideoEditorFlexForge {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FlexForge Integration Trait
// ============================================================================

impl FlexForgeIntegration for VideoEditorFlexForge {
    fn panel_id(&self) -> &str {
        "essentia_video_editor_plugin"
    }

    fn category(&self) -> FlexForgePanelCategory {
        FlexForgePanelCategory::Media
    }

    fn display_name(&self) -> &str {
        "Video Editor"
    }

    fn icon_glyph(&self) -> Option<&str> {
        Some("\u{E714}") // Video camera icon
    }

    fn priority(&self) -> u32 {
        1 // Highest priority in Media category
    }

    fn on_panel_activate(&mut self) {
        // Initialize GPU pipeline on activation
    }

    fn on_panel_deactivate(&mut self) {
        // Release GPU resources if streaming stops
        if self.stream_active
            && let Some(id) = self.stream_id
        {
            let _ = self.stop_stream(id);
        }
    }

    fn on_refresh(&mut self) -> bool {
        self.stream_active || self.project_modified
    }
}

// ============================================================================
// UI Configurable Trait
// ============================================================================

impl UiConfigurable for VideoEditorFlexForge {
    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new()
            // Resolution settings
            .with_field(
                ConfigField::select(
                    "resolution",
                    "Resolution",
                    vec![
                        String::from("1280x720"),
                        String::from("1920x1080"),
                        String::from("2560x1440"),
                        String::from("3840x2160"),
                    ],
                )
                .with_description("Default project resolution")
                .with_group("Display"),
            )
            .with_field(
                ConfigField::select(
                    "frame_rate",
                    "Frame Rate",
                    vec![
                        String::from("24"),
                        String::from("30"),
                        String::from("60"),
                        String::from("120"),
                    ],
                )
                .with_description("Timeline frame rate")
                .with_group("Display"),
            )
            .with_field(
                ConfigField::toggle("gpu_acceleration", "GPU Acceleration", true)
                    .with_description("Enable GPU-accelerated rendering")
                    .with_group("Performance"),
            )
            .with_field(
                ConfigField::number("preview_quality", "Preview Quality", 75.0, 10.0, 100.0)
                    .with_description("Preview render quality percentage")
                    .with_group("Performance"),
            )
            // Timeline settings
            .with_field(
                ConfigField::number(
                    "autosave_interval_sec",
                    "Auto-Save Interval (s)",
                    300.0,
                    30.0,
                    3600.0,
                )
                .with_description("Auto-save frequency in seconds")
                .with_group("Timeline"),
            )
            .with_field(
                ConfigField::number("undo_depth", "Undo History", 100.0, 10.0, 500.0)
                    .with_description("Maximum undo operations stored")
                    .with_group("Timeline"),
            )
            .with_field(
                ConfigField::toggle("snap_to_grid", "Snap to Grid", true)
                    .with_description("Snap timeline elements to grid")
                    .with_group("Timeline"),
            )
            // Export settings
            .with_field(
                ConfigField::select(
                    "export_codec",
                    "Export Codec",
                    vec![
                        String::from("h264"),
                        String::from("h265"),
                        String::from("av1"),
                        String::from("prores"),
                    ],
                )
                .with_description("Default export video codec")
                .with_group("Export"),
            )
            .with_field(
                ConfigField::number("export_bitrate_mbps", "Bitrate (Mbps)", 50.0, 1.0, 500.0)
                    .with_description("Export video bitrate")
                    .with_group("Export"),
            )
            .with_field(
                ConfigField::toggle("hardware_encoding", "Hardware Encoding", true)
                    .with_description("Use GPU hardware encoder")
                    .with_group("Export"),
            )
            // AI settings
            .with_field(
                ConfigField::toggle("ai_scene_detection", "AI Scene Detection", true)
                    .with_description("Automatic scene boundary detection")
                    .with_group("AI Features"),
            )
            .with_field(
                ConfigField::toggle("ai_color_grading", "AI Color Grading", false)
                    .with_description("AI-assisted color correction")
                    .with_group("AI Features"),
            )
    }

    fn on_config_changed(&mut self, key: &str, value: &str) -> Result<(), String> {
        let mut config = self.config.lock().map_err(|_| "Lock poisoned")?;

        match key {
            "resolution" => {
                let parts: Vec<&str> = value.split('x').collect();
                if parts.len() == 2 {
                    config.resolution_width = parts[0].parse().map_err(|_| "Invalid width")?;
                    config.resolution_height = parts[1].parse().map_err(|_| "Invalid height")?;
                }
                Ok(())
            },
            "frame_rate" => {
                config.frame_rate = value.parse().map_err(|_| "Invalid frame rate")?;
                Ok(())
            },
            "gpu_acceleration" => {
                config.gpu_acceleration = value == "true";
                Ok(())
            },
            "preview_quality" => {
                let v: f64 = value.parse().map_err(|_| "Invalid number")?;
                if !(10.0..=100.0).contains(&v) {
                    return Err("Preview quality must be between 10 and 100".to_string());
                }
                config.preview_quality = v as u32;
                Ok(())
            },
            "autosave_interval_sec" => {
                let v: f64 = value.parse().map_err(|_| "Invalid number")?;
                config.autosave_interval_sec = v as u32;
                Ok(())
            },
            "undo_depth" => {
                let v: f64 = value.parse().map_err(|_| "Invalid number")?;
                config.undo_depth = v as u32;
                Ok(())
            },
            "snap_to_grid" => {
                config.snap_to_grid = value == "true";
                Ok(())
            },
            "export_codec" => {
                config.export_codec = value.to_string();
                Ok(())
            },
            "export_bitrate_mbps" => {
                let v: f64 = value.parse().map_err(|_| "Invalid number")?;
                config.export_bitrate_mbps = v as u32;
                Ok(())
            },
            "hardware_encoding" => {
                config.hardware_encoding = value == "true";
                Ok(())
            },
            "ai_scene_detection" => {
                config.ai_scene_detection = value == "true";
                Ok(())
            },
            "ai_color_grading" => {
                config.ai_color_grading = value == "true";
                Ok(())
            },
            _ => Err(format!("Unknown configuration key: {key}")),
        }
    }

    fn apply_config(&mut self, config: &[(String, String)]) -> Result<(), String> {
        for (key, value) in config {
            self.on_config_changed(key, value)?;
        }
        Ok(())
    }

    fn get_current_config(&self) -> Vec<(String, String)> {
        let config = self.config.lock().unwrap_or_else(|p| p.into_inner());
        vec![
            (
                String::from("resolution"),
                format!("{}x{}", config.resolution_width, config.resolution_height),
            ),
            (String::from("frame_rate"), config.frame_rate.to_string()),
            (
                String::from("gpu_acceleration"),
                config.gpu_acceleration.to_string(),
            ),
            (
                String::from("preview_quality"),
                config.preview_quality.to_string(),
            ),
            (
                String::from("autosave_interval_sec"),
                config.autosave_interval_sec.to_string(),
            ),
            (String::from("undo_depth"), config.undo_depth.to_string()),
            (
                String::from("snap_to_grid"),
                config.snap_to_grid.to_string(),
            ),
            (String::from("export_codec"), config.export_codec.clone()),
            (
                String::from("export_bitrate_mbps"),
                config.export_bitrate_mbps.to_string(),
            ),
            (
                String::from("hardware_encoding"),
                config.hardware_encoding.to_string(),
            ),
            (
                String::from("ai_scene_detection"),
                config.ai_scene_detection.to_string(),
            ),
            (
                String::from("ai_color_grading"),
                config.ai_color_grading.to_string(),
            ),
        ]
    }

    fn reset_to_defaults(&mut self) {
        if let Ok(mut config) = self.config.lock() {
            *config = VideoEditorConfig::default();
        }
    }
}

// ============================================================================
// Editor Presentable Trait
// ============================================================================

impl EditorPresentable for VideoEditorFlexForge {
    fn editor_type(&self) -> &str {
        "video_timeline"
    }

    fn supported_content_types(&self) -> Vec<String> {
        vec![
            String::from("video/mp4"),
            String::from("video/webm"),
            String::from("video/quicktime"),
            String::from("video/x-matroska"),
            String::from("essentia/video-project"),
        ]
    }

    fn load_content(&mut self, content_id: &str, _content_type: &str) -> Result<(), String> {
        self.current_project = Some(content_id.to_string());
        self.project_modified = false;
        Ok(())
    }

    fn save_content(&self) -> Result<String, String> {
        self.current_project.clone().ok_or_else(|| "No project loaded".to_string())
    }

    fn has_unsaved_changes(&self) -> bool {
        self.project_modified
    }

    fn get_toolbar_actions(&self) -> Vec<EditorAction> {
        vec![
            EditorAction {
                id:       String::from("video_play"),
                label:    String::from("Play/Pause"),
                icon:     String::from("\u{E768}"),
                shortcut: Some(String::from("Space")),
                enabled:  self.current_project.is_some(),
            },
            EditorAction {
                id:       String::from("video_cut"),
                label:    String::from("Cut"),
                icon:     String::from("\u{E8C6}"),
                shortcut: Some(String::from("Ctrl+X")),
                enabled:  self.current_project.is_some(),
            },
            EditorAction {
                id:       String::from("video_add_track"),
                label:    String::from("Add Track"),
                icon:     String::from("\u{E710}"),
                shortcut: Some(String::from("Ctrl+Shift+T")),
                enabled:  self.current_project.is_some(),
            },
            EditorAction {
                id:       String::from("video_effects"),
                label:    String::from("Effects"),
                icon:     String::from("\u{E7AC}"),
                shortcut: Some(String::from("Ctrl+E")),
                enabled:  self.current_project.is_some(),
            },
            EditorAction {
                id:       String::from("video_export"),
                label:    String::from("Export"),
                icon:     String::from("\u{E898}"),
                shortcut: Some(String::from("Ctrl+Shift+E")),
                enabled:  self.current_project.is_some(),
            },
        ]
    }
}

// ============================================================================
// Streaming Capable Trait
// ============================================================================

impl StreamingCapable for VideoEditorFlexForge {
    fn is_streaming(&self) -> bool {
        self.stream_active
    }

    fn start_stream(&mut self) -> Result<u64, String> {
        if self.stream_active {
            return Err("Stream already active".to_string());
        }

        let stream_id = self.next_stream_id();
        self.stream_id = Some(stream_id);
        self.stream_active = true;

        Ok(stream_id)
    }

    fn stop_stream(&mut self, stream_id: u64) -> Result<(), String> {
        if !self.stream_active {
            return Err("No active stream".to_string());
        }

        if self.stream_id != Some(stream_id) {
            return Err("Invalid stream ID".to_string());
        }

        self.stream_active = false;
        self.stream_id = None;

        Ok(())
    }

    fn target_fps(&self) -> u32 {
        // Match project frame rate for preview streaming
        let config = self.config.lock().unwrap_or_else(|p| p.into_inner());
        config.frame_rate
    }

    fn render_frame(&mut self, stream_id: u64, delta_ms: f64) -> bool {
        if !self.stream_active || self.stream_id != Some(stream_id) {
            return false;
        }

        // Update playback position
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.playback_position_ms += delta_ms as u64;
            if metrics.playback_position_ms > metrics.timeline_duration_ms {
                metrics.playback_position_ms = 0; // Loop
            }
        }

        true
    }
}

#[cfg(all(test, feature = "full-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_panel_info() {
        let integration = VideoEditorFlexForge::new();
        assert_eq!(integration.panel_id(), "essentia_video_editor_plugin");
        assert_eq!(integration.category(), FlexForgePanelCategory::Media);
    }

    #[test]
    fn test_config_schema() {
        let integration = VideoEditorFlexForge::new();
        let schema = integration.config_schema();
        assert!(!schema.fields.is_empty());
        assert!(schema.fields.iter().any(|f| f.key == "resolution"));
    }

    #[test]
    fn test_editor_presentable() {
        let mut integration = VideoEditorFlexForge::new();

        assert!(integration.load_content("test_project.evp", "essentia/video-project").is_ok());
        assert!(!integration.has_unsaved_changes());

        let actions = integration.get_toolbar_actions();
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "video_play"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_streaming_lifecycle() {
        let mut integration = VideoEditorFlexForge::new();

        let stream_id = integration.start_stream().expect("should start streaming");
        assert!(integration.is_streaming());
        assert_eq!(integration.target_fps(), 60); // Default frame rate

        integration.stop_stream(stream_id).expect("should stop streaming");
        assert!(!integration.is_streaming());
    }

    #[test]
    fn test_config_validation() {
        let mut integration = VideoEditorFlexForge::new();

        // Valid preview quality
        assert!(integration.on_config_changed("preview_quality", "80").is_ok());

        // Invalid preview quality
        assert!(integration.on_config_changed("preview_quality", "150").is_err());
    }
}

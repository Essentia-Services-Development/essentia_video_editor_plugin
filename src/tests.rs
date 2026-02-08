use super::*;

#[test]
fn test_video_editor_plugin_creation() {
    let config = VideoEditorConfig::default();
    let plugin = VideoEditorPlugin::new(config);
    assert!(plugin.is_ok(), "VideoEditorPlugin should be created successfully");
}

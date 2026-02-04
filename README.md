# Essentia Video Editor Plugin

AI-enhanced video editing with GPU acceleration for the Essentia ecosystem.

## Features

- **Timeline Management**: Multi-track video/audio timeline
- **Asset Library**: Media asset organization and management
- **Effects Pipeline**: Real-time video effects processing
- **GPU Acceleration**: Hardware-accelerated rendering via essentia_gpu_accel_kernel

## Usage

```rust
use essentia_video_editor_plugin::{VideoEditorPlugin, VideoEditorConfig};

let plugin = VideoEditorPlugin::default();
let timeline = plugin.create_timeline(30.0)?; // 30 FPS
plugin.add_video_clip(&timeline.id, video_asset)?;
```

## SSOP Compliance

This plugin is fully SSOP-compliant (std-only, zero third-party dependencies).

## License

MIT

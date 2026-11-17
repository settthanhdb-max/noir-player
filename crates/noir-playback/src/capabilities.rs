//! Backend capability reporting.
//!
//! Capabilities describe what the *actual* engine can do. The settings UI
//! uses them to hide unsupported options instead of showing dead controls.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capabilities {
    /// The engine can decode and render video at all.
    pub video: bool,
    pub audio: bool,
    pub hardware_decoding: bool,
    pub subtitle_rendering: bool,
    pub network_streaming: bool,
    pub frame_stepping: bool,
    pub chapter_navigation: bool,
    pub ab_repeat: bool,
    pub screenshot: bool,
    /// Audio DSP (equalizer, normalization) is applied through real filters.
    pub audio_dsp: bool,
    pub video_equalizer: bool,
    /// Human-readable backend name, e.g. "libmpv 2.3" or "mock".
    pub backend_name: String,
    /// Free-form notes shown in the capability report, e.g. missing libraries.
    pub notes: Vec<String>,
}

impl Capabilities {
    /// Capabilities of the mock engine: honest about doing nothing.
    pub fn mock() -> Self {
        Self {
            video: false,
            audio: false,
            hardware_decoding: false,
            subtitle_rendering: false,
            network_streaming: false,
            frame_stepping: false,
            chapter_navigation: false,
            ab_repeat: false,
            screenshot: false,
            audio_dsp: false,
            video_equalizer: false,
            backend_name: "mock (no native backend available)".into(),
            notes: vec![
                "Built without the libmpv backend or libmpv failed to load.".into(),
                "Playback controls are wired but will not produce media output.".into(),
            ],
        }
    }

    /// Capabilities of a full libmpv backend.
    pub fn mpv(version: &str) -> Self {
        Self {
            video: true,
            audio: true,
            hardware_decoding: true,
            subtitle_rendering: true,
            network_streaming: true,
            frame_stepping: true,
            chapter_navigation: true,
            ab_repeat: true,
            screenshot: true,
            audio_dsp: true,
            video_equalizer: true,
            backend_name: format!("libmpv {version}"),
            notes: Vec::new(),
        }
    }
}

//! Typed settings schema, organized by category.

pub mod advanced;
pub mod appearance;
pub mod audio;
pub mod general;
pub mod input;
pub mod network;
pub mod playback;
pub mod playlist;
pub mod subtitles;
pub mod video;

use serde::{Deserialize, Serialize};

pub use advanced::AdvancedSettings;
pub use appearance::AppearanceSettings;
pub use audio::AudioSettings;
pub use general::GeneralSettings;
pub use input::InputSettings;
pub use network::NetworkSettings;
pub use playback::PlaybackSettings;
pub use playlist::PlaylistSettings;
pub use subtitles::SubtitleSettings;
pub use video::VideoSettings;

/// Root settings object. Every category deserializes with `#[serde(default)]`
/// so that adding new settings never breaks older configuration files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings {
    pub version: u32,
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub appearance: AppearanceSettings,
    #[serde(default)]
    pub playback: PlaybackSettings,
    #[serde(default)]
    pub video: VideoSettings,
    #[serde(default)]
    pub audio: AudioSettings,
    #[serde(default)]
    pub subtitles: SubtitleSettings,
    #[serde(default)]
    pub network: NetworkSettings,
    #[serde(default)]
    pub playlist: PlaylistSettings,
    #[serde(default)]
    pub input: InputSettings,
    #[serde(default)]
    pub advanced: AdvancedSettings,
}

impl Settings {
    pub fn defaults() -> Self {
        Self {
            version: crate::SETTINGS_VERSION,
            general: GeneralSettings::default(),
            appearance: AppearanceSettings::default(),
            playback: PlaybackSettings::default(),
            video: VideoSettings::default(),
            audio: AudioSettings::default(),
            subtitles: SubtitleSettings::default(),
            network: NetworkSettings::default(),
            playlist: PlaylistSettings::default(),
            input: InputSettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }

    /// Iterate every category descriptor group in display order.
    pub fn describe_all(&mut self) -> Vec<(&'static str, Vec<crate::value::SettingEntry<'_>>)> {
        vec![
            ("general", self.general.describe()),
            ("appearance", self.appearance.describe()),
            ("playback", self.playback.describe()),
            ("video", self.video.describe()),
            ("audio", self.audio.describe()),
            ("subtitles", self.subtitles.describe()),
            ("network", self.network.describe()),
            ("playlist", self.playlist.describe()),
            ("input", self.input.describe()),
            ("advanced", self.advanced.describe()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_roundtrip_preserves_values() {
        let mut s = Settings::defaults();
        s.playback.default_volume = 77;
        s.appearance.ui_scale = 1.25;
        let text = toml::to_string_pretty(&s).unwrap();
        let back: Settings = toml::from_str(&text).unwrap();
        assert_eq!(back.playback.default_volume, 77);
        assert!((back.appearance.ui_scale - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let s: Settings = toml::from_str("version = 2").unwrap();
        assert_eq!(s.playback.default_volume, 100);
        assert_eq!(s.general.language, 0);
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let s: Settings = toml::from_str("version = 2\n[general]\nobsolete_key = 1").unwrap();
        assert_eq!(s.general.language, 0);
    }
}

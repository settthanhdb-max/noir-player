//! Category 9 — Keyboard & Mouse, including the extensible shortcut registry.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Every user-triggerable playback/UI action that can be bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionBinding {
    PlayPause,
    SeekBackward,
    SeekForward,
    LongSeekBackward,
    LongSeekForward,
    VolumeUp,
    VolumeDown,
    Mute,
    Fullscreen,
    OpenFile,
    OpenFolder,
    PlaylistPanel,
    SubtitleMenu,
    AudioTrackMenu,
    SpeedDown,
    SpeedUp,
    SpeedReset,
    FrameStepBackward,
    FrameStepForward,
    PreviousMedia,
    NextMedia,
    ChapterPrevious,
    ChapterNext,
    Screenshot,
    InfoOverlay,
    AbRepeatMark,
    Stop,
    ExitFullscreenCloseDialog,
}

impl ActionBinding {
    pub const ALL: [ActionBinding; 28] = [
        Self::PlayPause,
        Self::SeekBackward,
        Self::SeekForward,
        Self::LongSeekBackward,
        Self::LongSeekForward,
        Self::VolumeUp,
        Self::VolumeDown,
        Self::Mute,
        Self::Fullscreen,
        Self::OpenFile,
        Self::OpenFolder,
        Self::PlaylistPanel,
        Self::SubtitleMenu,
        Self::AudioTrackMenu,
        Self::SpeedDown,
        Self::SpeedUp,
        Self::SpeedReset,
        Self::FrameStepBackward,
        Self::FrameStepForward,
        Self::PreviousMedia,
        Self::NextMedia,
        Self::ChapterPrevious,
        Self::ChapterNext,
        Self::Screenshot,
        Self::InfoOverlay,
        Self::AbRepeatMark,
        Self::Stop,
        Self::ExitFullscreenCloseDialog,
    ];

    /// Stable identifier used in serialized profiles.
    pub fn id(self) -> &'static str {
        match self {
            Self::PlayPause => "play_pause",
            Self::SeekBackward => "seek_backward",
            Self::SeekForward => "seek_forward",
            Self::LongSeekBackward => "long_seek_backward",
            Self::LongSeekForward => "long_seek_forward",
            Self::VolumeUp => "volume_up",
            Self::VolumeDown => "volume_down",
            Self::Mute => "mute",
            Self::Fullscreen => "fullscreen",
            Self::OpenFile => "open_file",
            Self::OpenFolder => "open_folder",
            Self::PlaylistPanel => "playlist_panel",
            Self::SubtitleMenu => "subtitle_menu",
            Self::AudioTrackMenu => "audio_track_menu",
            Self::SpeedDown => "speed_down",
            Self::SpeedUp => "speed_up",
            Self::SpeedReset => "speed_reset",
            Self::FrameStepBackward => "frame_step_backward",
            Self::FrameStepForward => "frame_step_forward",
            Self::PreviousMedia => "previous_media",
            Self::NextMedia => "next_media",
            Self::ChapterPrevious => "chapter_previous",
            Self::ChapterNext => "chapter_next",
            Self::Screenshot => "screenshot",
            Self::InfoOverlay => "info_overlay",
            Self::AbRepeatMark => "ab_repeat_mark",
            Self::Stop => "stop",
            Self::ExitFullscreenCloseDialog => "exit_fullscreen_close",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|a| a.id() == id)
    }

    pub fn default_label(self) -> &'static str {
        match self {
            Self::PlayPause => "Play / Pause",
            Self::SeekBackward => "Seek backward",
            Self::SeekForward => "Seek forward",
            Self::LongSeekBackward => "Long seek backward",
            Self::LongSeekForward => "Long seek forward",
            Self::VolumeUp => "Volume up",
            Self::VolumeDown => "Volume down",
            Self::Mute => "Mute",
            Self::Fullscreen => "Fullscreen",
            Self::OpenFile => "Open file",
            Self::OpenFolder => "Open folder",
            Self::PlaylistPanel => "Playlist panel",
            Self::SubtitleMenu => "Subtitle menu",
            Self::AudioTrackMenu => "Audio track menu",
            Self::SpeedDown => "Decrease speed",
            Self::SpeedUp => "Increase speed",
            Self::SpeedReset => "Reset speed",
            Self::FrameStepBackward => "Frame step backward",
            Self::FrameStepForward => "Frame step forward",
            Self::PreviousMedia => "Previous media",
            Self::NextMedia => "Next media",
            Self::ChapterPrevious => "Previous chapter",
            Self::ChapterNext => "Next chapter",
            Self::Screenshot => "Screenshot",
            Self::InfoOverlay => "Info overlay",
            Self::AbRepeatMark => "A-B repeat mark",
            Self::Stop => "Stop",
            Self::ExitFullscreenCloseDialog => "Exit fullscreen / close dialog",
        }
    }
}

/// Modifier flags for a key combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        shift: false,
        alt: false,
    };
    pub const SHIFT: Self = Self {
        ctrl: false,
        shift: true,
        alt: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        shift: false,
        alt: true,
    };

    pub fn ctrl() -> Self {
        Self::CTRL
    }

    pub fn label(self) -> String {
        let mut s = String::new();
        if self.ctrl {
            s.push_str("Ctrl+");
        }
        if self.alt {
            s.push_str("Alt+");
        }
        if self.shift {
            s.push_str("Shift+");
        }
        s
    }
}

/// A keyboard combination: optional modifiers plus a key name.
/// Key names follow egui's `Key` variants (e.g. "ArrowLeft") or single
/// characters (e.g. "a", "[", "0").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCombo {
    #[serde(default)]
    pub modifiers: Modifiers,
    pub key: String,
}

impl fmt::Display for KeyCombo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.modifiers.label(), self.key)
    }
}

/// One binding: action plus its key combination.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub action: ActionBinding,
    pub combo: KeyCombo,
}

/// The full input configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputSettings {
    pub bindings: Vec<KeyBinding>,
    /// Mouse wheel over video: 0 = volume, 1 = seek, 2 = speed.
    pub wheel_action: usize,
    /// Wheel seek step in seconds when the wheel is bound to seeking.
    pub wheel_seek_s: f32,
    /// Double-click over video: 0 = fullscreen, 1 = play/pause, 2 = nothing.
    pub double_click_action: usize,
    pub gestures_enabled: bool,
    pub gesture_sensitivity: f32,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            bindings: default_bindings(),
            wheel_action: 0,
            wheel_seek_s: 5.0,
            double_click_action: 0,
            gestures_enabled: false,
            gesture_sensitivity: 1.0,
        }
    }
}

/// Default keymap from the NOIR Player specification.
pub fn default_bindings() -> Vec<KeyBinding> {
    use ActionBinding::*;
    let k = |mods: Modifiers, key: &str, action: ActionBinding| KeyBinding {
        action,
        combo: KeyCombo {
            modifiers: mods,
            key: key.to_string(),
        },
    };
    vec![
        k(Modifiers::NONE, "Space", PlayPause),
        k(Modifiers::NONE, "ArrowLeft", SeekBackward),
        k(Modifiers::NONE, "ArrowRight", SeekForward),
        k(Modifiers::NONE, "ArrowUp", VolumeUp),
        k(Modifiers::NONE, "ArrowDown", VolumeDown),
        k(Modifiers::NONE, "f", Fullscreen),
        k(Modifiers::NONE, "m", Mute),
        k(Modifiers::NONE, "o", OpenFile),
        k(Modifiers::NONE, "p", PlaylistPanel),
        k(Modifiers::NONE, "s", SubtitleMenu),
        k(Modifiers::NONE, "a", AudioTrackMenu),
        k(Modifiers::NONE, "[", SpeedDown),
        k(Modifiers::NONE, "]", SpeedUp),
        k(Modifiers::NONE, "0", SpeedReset),
        k(Modifiers::NONE, ",", FrameStepBackward),
        k(Modifiers::NONE, ".", FrameStepForward),
        k(Modifiers::NONE, "Escape", ExitFullscreenCloseDialog),
        k(Modifiers::NONE, "n", NextMedia),
        k(Modifiers::NONE, "b", PreviousMedia),
        k(Modifiers::NONE, "i", InfoOverlay),
        k(Modifiers::NONE, "c", Screenshot),
        k(Modifiers::NONE, "l", ChapterNext),
        k(Modifiers::NONE, "j", ChapterPrevious),
        k(Modifiers::ctrl(), "o", OpenFolder),
        k(Modifiers::ctrl(), "r", AbRepeatMark),
        k(Modifiers::ctrl(), "s", Screenshot),
    ]
}

/// The extensible shortcut registry: lookup, conflict detection, reset.
#[derive(Debug, Clone, Default)]
pub struct ShortcutRegistry {
    bindings: Vec<KeyBinding>,
}

impl ShortcutRegistry {
    pub fn new(bindings: Vec<KeyBinding>) -> Self {
        Self { bindings }
    }

    pub fn bindings(&self) -> &[KeyBinding] {
        &self.bindings
    }

    pub fn set_bindings(&mut self, bindings: Vec<KeyBinding>) {
        self.bindings = bindings;
    }

    /// Resolve the action bound to a pressed combination.
    pub fn lookup(&self, modifiers: Modifiers, key: &str) -> Option<ActionBinding> {
        self.bindings
            .iter()
            .find(|b| b.combo.key.eq_ignore_ascii_case(key) && b.combo.modifiers == modifiers)
            .map(|b| b.action)
    }

    /// Combinations assigned to more than one action. Multiple combinations
    /// per action are legitimate; a shared combination is a real conflict.
    pub fn conflicts(&self) -> Vec<(KeyCombo, Vec<ActionBinding>)> {
        let mut out: Vec<(KeyCombo, Vec<ActionBinding>)> = Vec::new();
        for b in &self.bindings {
            if out.iter().any(|(c, _)| c == &b.combo) {
                continue;
            }
            let mut actions: Vec<ActionBinding> = self
                .bindings
                .iter()
                .filter(|other| other.combo == b.combo)
                .map(|other| other.action)
                .collect();
            actions.dedup();
            if actions.len() > 1 {
                out.push((b.combo.clone(), actions));
            }
        }
        out
    }

    /// Assign a new combo to an action, removing duplicate assignments of the
    /// same combo from other actions. Returns the previous holder, if any.
    pub fn assign(&mut self, action: ActionBinding, combo: KeyCombo) -> Option<ActionBinding> {
        let previous = self
            .bindings
            .iter()
            .find(|b| b.combo == combo && b.action != action)
            .map(|b| b.action);
        self.bindings.retain(|b| b.combo != combo);
        self.bindings.retain(|b| b.action != action);
        self.bindings.push(KeyBinding { action, combo });
        previous
    }

    pub fn unassign(&mut self, action: ActionBinding) {
        self.bindings.retain(|b| b.action != action);
    }

    pub fn reset_to_defaults(&mut self) {
        self.bindings = default_bindings();
    }
}

impl InputSettings {
    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "input.wheel_action",
                "Mouse wheel behavior",
                "What the wheel does over the video viewport.",
                SettingKind::Dropdown {
                    options: ["Volume", "Seek", "Speed"].to_vec(),
                },
                SettingValue::Choice(&mut self.wheel_action),
            ),
            SettingEntry::new(
                "input.wheel_seek_s",
                "Wheel seek step",
                "Seconds skipped per wheel notch when seeking.",
                SettingKind::Slider {
                    min: 1.0,
                    max: 60.0,
                    step: 1.0,
                    unit: "s",
                },
                SettingValue::F32(&mut self.wheel_seek_s),
            ),
            SettingEntry::new(
                "input.double_click_action",
                "Double-click behavior",
                "Double-click action over the video viewport.",
                SettingKind::Dropdown {
                    options: ["Toggle fullscreen", "Play / pause", "Nothing"].to_vec(),
                },
                SettingValue::Choice(&mut self.double_click_action),
            ),
            SettingEntry::new(
                "input.gestures_enabled",
                "Mouse gestures",
                "Enable optional mouse gestures.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.gestures_enabled),
            ),
            SettingEntry::new(
                "input.gesture_sensitivity",
                "Gesture sensitivity",
                "Gesture recognition distance threshold.",
                SettingKind::Slider {
                    min: 0.25,
                    max: 4.0,
                    step: 0.25,
                    unit: "x",
                },
                SettingValue::F32(&mut self.gesture_sensitivity),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_have_no_conflicts() {
        let reg = ShortcutRegistry::new(default_bindings());
        assert!(
            reg.conflicts().is_empty(),
            "conflicts: {:?}",
            reg.conflicts()
        );
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let reg = ShortcutRegistry::new(default_bindings());
        assert_eq!(
            reg.lookup(Modifiers::NONE, "space"),
            Some(ActionBinding::PlayPause)
        );
        assert_eq!(
            reg.lookup(Modifiers::NONE, "F"),
            Some(ActionBinding::Fullscreen)
        );
        assert_eq!(reg.lookup(Modifiers::NONE, "x"), None);
    }

    #[test]
    fn assign_steals_combo() {
        let mut reg = ShortcutRegistry::new(default_bindings());
        let combo = KeyCombo {
            modifiers: Modifiers::NONE,
            key: "p".into(),
        };
        let prev = reg.assign(ActionBinding::OpenFile, combo.clone());
        assert_eq!(prev, Some(ActionBinding::PlaylistPanel));
        assert_eq!(
            reg.lookup(Modifiers::NONE, "p"),
            Some(ActionBinding::OpenFile)
        );
        assert!(
            reg.lookup(Modifiers::NONE, "o").is_none()
                || reg.lookup(Modifiers::NONE, "o") == Some(ActionBinding::OpenFile)
        );
    }

    #[test]
    fn unassign_and_reset() {
        let mut reg = ShortcutRegistry::new(default_bindings());
        reg.unassign(ActionBinding::Mute);
        assert_eq!(reg.lookup(Modifiers::NONE, "m"), None);
        reg.reset_to_defaults();
        assert_eq!(reg.lookup(Modifiers::NONE, "m"), Some(ActionBinding::Mute));
    }

    #[test]
    fn action_ids_roundtrip() {
        for a in ActionBinding::ALL {
            assert_eq!(ActionBinding::from_id(a.id()), Some(a));
        }
    }

    #[test]
    fn bindings_serialize() {
        let json = serde_json::to_string(&default_bindings()).unwrap();
        let back: Vec<KeyBinding> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), default_bindings().len());
    }
}

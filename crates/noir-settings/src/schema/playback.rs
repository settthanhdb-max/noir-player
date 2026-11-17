//! Category 3 — Playback.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const MAX_VOLUME_POLICIES: [&str; 3] = ["100% hard limit", "Allow boost to 130%", "Unlimited"];
pub const COMPLETION_ACTIONS: [&str; 4] = ["Stop", "Play next", "Repeat file", "Exit application"];
pub const SHUFFLE_BEHAVIORS: [&str; 3] =
    ["Random each time", "Shuffled queue", "Random within folder"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackSettings {
    pub default_speed: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub speed_increment: f32,
    pub default_volume: u32,
    /// Index into [`MAX_VOLUME_POLICIES`].
    pub max_volume_policy: usize,
    pub remember_volume: bool,
    pub auto_resume: bool,
    pub resume_threshold_s: f32,
    pub seek_backward_s: f32,
    pub seek_forward_s: f32,
    pub long_seek_s: f32,
    pub frame_step_enabled: bool,
    /// Index into [`COMPLETION_ACTIONS`].
    pub completion_action: usize,
    pub auto_play_next: bool,
    pub repeat_file: bool,
    pub repeat_playlist: bool,
    pub shuffle: bool,
    /// Index into [`SHUFFLE_BEHAVIORS`].
    pub shuffle_behavior: usize,
    pub pause_when_minimized: bool,
    pub pause_on_focus_loss: bool,
    pub resume_on_focus: bool,
    pub ab_repeat_enabled: bool,
    pub chapter_navigation: bool,
    pub bookmarks_enabled: bool,
    pub record_history: bool,
    pub remember_position_per_file: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            default_speed: 1.0,
            min_speed: 0.25,
            max_speed: 4.0,
            speed_increment: 0.1,
            default_volume: 100,
            max_volume_policy: 1,
            remember_volume: true,
            auto_resume: true,
            resume_threshold_s: 5.0,
            seek_backward_s: 5.0,
            seek_forward_s: 5.0,
            long_seek_s: 30.0,
            frame_step_enabled: true,
            completion_action: 1,
            auto_play_next: true,
            repeat_file: false,
            repeat_playlist: false,
            shuffle: false,
            shuffle_behavior: 1,
            pause_when_minimized: false,
            pause_on_focus_loss: false,
            resume_on_focus: true,
            ab_repeat_enabled: true,
            chapter_navigation: true,
            bookmarks_enabled: true,
            record_history: true,
            remember_position_per_file: true,
        }
    }
}

impl PlaybackSettings {
    /// Clamp the volume to the configured maximum policy.
    pub fn clamp_volume(&self, volume: u32) -> u32 {
        match self.max_volume_policy {
            0 => volume.min(100),
            1 => volume.min(130),
            _ => volume,
        }
    }

    /// Clamp a speed request into the configured speed range.
    pub fn clamp_speed(&self, speed: f32) -> f32 {
        speed.clamp(self.min_speed, self.max_speed)
    }

    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "playback.default_speed",
                "Default playback speed",
                "Speed applied when a new file starts.",
                SettingKind::Slider {
                    min: 0.25,
                    max: 4.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.default_speed),
            ),
            SettingEntry::new(
                "playback.min_speed",
                "Minimum playback speed",
                "Lower bound of the speed control.",
                SettingKind::Number {
                    min: 0.1,
                    max: 1.0,
                    step: 0.05,
                },
                SettingValue::F32(&mut self.min_speed),
            ),
            SettingEntry::new(
                "playback.max_speed",
                "Maximum playback speed",
                "Upper bound of the speed control.",
                SettingKind::Number {
                    min: 1.0,
                    max: 8.0,
                    step: 0.25,
                },
                SettingValue::F32(&mut self.max_speed),
            ),
            SettingEntry::new(
                "playback.speed_increment",
                "Speed adjustment increment",
                "Step used by the speed up/down actions.",
                SettingKind::Slider {
                    min: 0.05,
                    max: 0.5,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.speed_increment),
            ),
            SettingEntry::new(
                "playback.default_volume",
                "Default volume",
                "Volume applied on startup, in percent.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 130.0,
                    step: 1.0,
                    unit: "%",
                },
                SettingValue::U32(&mut self.default_volume),
            ),
            SettingEntry::new(
                "playback.max_volume_policy",
                "Maximum volume policy",
                "How loud the player is allowed to get.",
                SettingKind::Dropdown {
                    options: MAX_VOLUME_POLICIES.to_vec(),
                },
                SettingValue::Choice(&mut self.max_volume_policy),
            ),
            SettingEntry::new(
                "playback.remember_volume",
                "Remember volume",
                "Restore the last used volume on startup.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remember_volume),
            ),
            SettingEntry::new(
                "playback.auto_resume",
                "Automatically resume",
                "Resume files at their last known position.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_resume),
            ),
            SettingEntry::new(
                "playback.resume_threshold_s",
                "Resume threshold",
                "Only resume when the saved position is beyond this many seconds.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 60.0,
                    step: 1.0,
                    unit: "s",
                },
                SettingValue::F32(&mut self.resume_threshold_s),
            ),
            SettingEntry::new(
                "playback.seek_backward_s",
                "Seek backward interval",
                "Seconds skipped by the seek-backward action.",
                SettingKind::Slider {
                    min: 1.0,
                    max: 60.0,
                    step: 1.0,
                    unit: "s",
                },
                SettingValue::F32(&mut self.seek_backward_s),
            ),
            SettingEntry::new(
                "playback.seek_forward_s",
                "Seek forward interval",
                "Seconds skipped by the seek-forward action.",
                SettingKind::Slider {
                    min: 1.0,
                    max: 60.0,
                    step: 1.0,
                    unit: "s",
                },
                SettingValue::F32(&mut self.seek_forward_s),
            ),
            SettingEntry::new(
                "playback.long_seek_s",
                "Long seek interval",
                "Seconds skipped by the long-seek action.",
                SettingKind::Slider {
                    min: 10.0,
                    max: 300.0,
                    step: 10.0,
                    unit: "s",
                },
                SettingValue::F32(&mut self.long_seek_s),
            ),
            SettingEntry::new(
                "playback.frame_step_enabled",
                "Frame-by-frame navigation",
                "Enable frame stepping with , and . actions.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.frame_step_enabled),
            ),
            SettingEntry::new(
                "playback.completion_action",
                "Playback completion behavior",
                "What happens when a file reaches its end.",
                SettingKind::Dropdown {
                    options: COMPLETION_ACTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.completion_action),
            ),
            SettingEntry::new(
                "playback.auto_play_next",
                "Automatically play next file",
                "Continue with the next playlist entry when a file ends.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_play_next),
            ),
            SettingEntry::new(
                "playback.repeat_file",
                "Repeat current file",
                "Loop the current file.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.repeat_file),
            ),
            SettingEntry::new(
                "playback.repeat_playlist",
                "Repeat playlist",
                "Restart the playlist after the last entry.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.repeat_playlist),
            ),
            SettingEntry::new(
                "playback.shuffle",
                "Random playback",
                "Play playlist entries in random order.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.shuffle),
            ),
            SettingEntry::new(
                "playback.shuffle_behavior",
                "Shuffle behavior",
                "How the random order is produced.",
                SettingKind::Dropdown {
                    options: SHUFFLE_BEHAVIORS.to_vec(),
                },
                SettingValue::Choice(&mut self.shuffle_behavior),
            ),
            SettingEntry::new(
                "playback.pause_when_minimized",
                "Pause when minimized",
                "Pause playback when the window is minimized.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.pause_when_minimized),
            ),
            SettingEntry::new(
                "playback.pause_on_focus_loss",
                "Pause when focus is lost",
                "Pause playback when another window takes focus.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.pause_on_focus_loss),
            ),
            SettingEntry::new(
                "playback.resume_on_focus",
                "Restore playback on focus",
                "Resume playback when the window regains focus.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.resume_on_focus),
            ),
            SettingEntry::new(
                "playback.ab_repeat_enabled",
                "A-B repeat",
                "Enable A-B loop markers.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.ab_repeat_enabled),
            ),
            SettingEntry::new(
                "playback.chapter_navigation",
                "Chapter navigation",
                "Enable chapter list and next/previous chapter actions.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.chapter_navigation),
            ),
            SettingEntry::new(
                "playback.bookmarks_enabled",
                "Bookmark management",
                "Enable time bookmarks for the current file.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.bookmarks_enabled),
            ),
            SettingEntry::new(
                "playback.record_history",
                "Playback history",
                "Record played files in history. You stay in control of your data.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.record_history),
            ),
            SettingEntry::new(
                "playback.remember_position_per_file",
                "Remember position per file",
                "Store the last position of each file independently.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remember_position_per_file),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_policy_clamps() {
        let mut p = PlaybackSettings::default();
        p.max_volume_policy = 0;
        assert_eq!(p.clamp_volume(130), 100);
        p.max_volume_policy = 1;
        assert_eq!(p.clamp_volume(150), 130);
        p.max_volume_policy = 2;
        assert_eq!(p.clamp_volume(500), 500);
    }

    #[test]
    fn speed_clamps() {
        let p = PlaybackSettings::default();
        assert!((p.clamp_speed(10.0) - 4.0).abs() < f32::EPSILON);
        assert!((p.clamp_speed(0.01) - 0.25).abs() < f32::EPSILON);
    }
}

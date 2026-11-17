//! Category 1 — General.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const LANGUAGE_OPTIONS: [&str; 2] = ["English", "Tiếng Việt"];
pub const DATETIME_OPTIONS: [&str; 3] = ["ISO 8601", "Locale", "24h"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    pub start_with_windows: bool,
    pub start_minimized: bool,
    pub restore_session: bool,
    pub resume_playback: bool,
    pub confirm_exit: bool,
    pub tray_enabled: bool,
    pub always_on_top: bool,
    pub remember_window_position: bool,
    pub remember_window_size: bool,
    pub start_fullscreen: bool,
    pub open_recent_on_start: bool,
    pub prevent_sleep: bool,
    pub confirm_clear_history: bool,
    pub default_media_dir: String,
    pub screenshot_dir: String,
    /// Index into [`LANGUAGE_OPTIONS`].
    pub language: usize,
    /// Index into [`DATETIME_OPTIONS`].
    pub datetime_format: usize,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            start_minimized: false,
            restore_session: true,
            resume_playback: true,
            confirm_exit: false,
            tray_enabled: false,
            always_on_top: false,
            remember_window_position: true,
            remember_window_size: true,
            start_fullscreen: false,
            open_recent_on_start: false,
            prevent_sleep: true,
            confirm_clear_history: true,
            default_media_dir: String::new(),
            screenshot_dir: String::new(),
            language: 0,
            datetime_format: 0,
        }
    }
}

impl GeneralSettings {
    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "general.start_with_windows",
                "Start with Windows",
                "Launch NOIR Player when the system starts.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.start_with_windows),
            ),
            SettingEntry::new(
                "general.start_minimized",
                "Start minimized",
                "Start minimized to the taskbar or tray.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.start_minimized),
            ),
            SettingEntry::new(
                "general.restore_session",
                "Restore previous session",
                "Reopen the last playlist and window layout on startup.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.restore_session),
            ),
            SettingEntry::new(
                "general.resume_playback",
                "Resume previous playback",
                "Continue the last file from its saved position.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.resume_playback),
            ),
            SettingEntry::new(
                "general.confirm_exit",
                "Confirm exit",
                "Ask before closing while media is playing.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.confirm_exit),
            ),
            SettingEntry::new(
                "general.tray_enabled",
                "Keep running in tray",
                "Keep the application running in the system tray after closing the window.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.tray_enabled),
            ),
            SettingEntry::new(
                "general.always_on_top",
                "Always on top",
                "Keep the main window above other windows.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.always_on_top),
            ),
            SettingEntry::new(
                "general.remember_window_position",
                "Remember window position",
                "Restore the last window position.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remember_window_position),
            ),
            SettingEntry::new(
                "general.remember_window_size",
                "Remember window size",
                "Restore the last window size.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remember_window_size),
            ),
            SettingEntry::new(
                "general.start_fullscreen",
                "Start in fullscreen",
                "Open the player in fullscreen mode.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.start_fullscreen),
            ),
            SettingEntry::new(
                "general.open_recent_on_start",
                "Open recently played",
                "Automatically open the most recently played media on startup.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.open_recent_on_start),
            ),
            SettingEntry::new(
                "general.prevent_sleep",
                "Prevent system sleep",
                "Inhibit system sleep while media is playing.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.prevent_sleep),
            ),
            SettingEntry::new(
                "general.confirm_clear_history",
                "Confirm before clearing history",
                "Ask for confirmation before playback history is erased.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.confirm_clear_history),
            ),
            SettingEntry::new(
                "general.default_media_dir",
                "Default media directory",
                "Initial directory shown in the open-file dialog.",
                SettingKind::Directory,
                SettingValue::Text(&mut self.default_media_dir),
            ),
            SettingEntry::new(
                "general.screenshot_dir",
                "Default screenshot directory",
                "Where screenshots are saved. Empty uses the Pictures folder.",
                SettingKind::Directory,
                SettingValue::Text(&mut self.screenshot_dir),
            ),
            SettingEntry::new(
                "general.language",
                "Language",
                "Interface language.",
                SettingKind::Dropdown {
                    options: LANGUAGE_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.language),
            )
            .restart(),
            SettingEntry::new(
                "general.datetime_format",
                "Date and time format",
                "How timestamps are displayed.",
                SettingKind::Dropdown {
                    options: DATETIME_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.datetime_format),
            ),
        ]
    }
}

//! Category 8 — Playlist.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const SORT_OPTIONS: [&str; 5] = ["Name", "Date added", "Duration", "File type", "Last played"];
pub const DOUBLE_CLICK_ACTIONS: [&str; 3] = ["Play", "Play and close panel", "Preview"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaylistSettings {
    /// Index into [`SORT_OPTIONS`].
    pub default_sort: usize,
    pub auto_add_opened: bool,
    pub remove_missing: bool,
    pub remember_order: bool,
    /// Index into repeat options: off/all/one.
    pub repeat_mode: usize,
    pub shuffle: bool,
    pub show_duration: bool,
    pub show_media_type: bool,
    pub show_file_size: bool,
    /// Index into [`DOUBLE_CLICK_ACTIONS`].
    pub double_click_action: usize,
    pub auto_play_selected: bool,
    pub save_changes: bool,
    pub persistent_queue: bool,
}

impl Default for PlaylistSettings {
    fn default() -> Self {
        Self {
            default_sort: 0,
            auto_add_opened: true,
            remove_missing: true,
            remember_order: true,
            repeat_mode: 0,
            shuffle: false,
            show_duration: true,
            show_media_type: true,
            show_file_size: false,
            double_click_action: 0,
            auto_play_selected: true,
            save_changes: true,
            persistent_queue: true,
        }
    }
}

impl PlaylistSettings {
    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "playlist.default_sort",
                "Default playlist sorting",
                "Sort order applied to new playlists.",
                SettingKind::Dropdown {
                    options: SORT_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.default_sort),
            ),
            SettingEntry::new(
                "playlist.auto_add_opened",
                "Automatically add opened files",
                "Files opened directly are appended to the queue.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_add_opened),
            ),
            SettingEntry::new(
                "playlist.remove_missing",
                "Remove missing files",
                "Drop entries whose files no longer exist.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remove_missing),
            ),
            SettingEntry::new(
                "playlist.remember_order",
                "Remember playlist order",
                "Persist manual reordering between sessions.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.remember_order),
            ),
            SettingEntry::new(
                "playlist.repeat_mode",
                "Repeat mode",
                "Off, repeat all, or repeat one.",
                SettingKind::Dropdown {
                    options: ["Off", "All", "One"].to_vec(),
                },
                SettingValue::Choice(&mut self.repeat_mode),
            ),
            SettingEntry::new(
                "playlist.shuffle",
                "Shuffle mode",
                "Random playback order.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.shuffle),
            ),
            SettingEntry::new(
                "playlist.show_duration",
                "Display duration",
                "Show track duration in the playlist.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_duration),
            ),
            SettingEntry::new(
                "playlist.show_media_type",
                "Display media type",
                "Show the file format column.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_media_type),
            ),
            SettingEntry::new(
                "playlist.show_file_size",
                "Display file size",
                "Show file sizes in the playlist.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_file_size),
            ),
            SettingEntry::new(
                "playlist.double_click_action",
                "Double-click behavior",
                "What double-clicking a playlist entry does.",
                SettingKind::Dropdown {
                    options: DOUBLE_CLICK_ACTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.double_click_action),
            ),
            SettingEntry::new(
                "playlist.auto_play_selected",
                "Automatically play selected media",
                "Start playback when an entry is selected.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_play_selected),
            ),
            SettingEntry::new(
                "playlist.save_changes",
                "Save playlist changes",
                "Persist playlist edits automatically.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.save_changes),
            ),
            SettingEntry::new(
                "playlist.persistent_queue",
                "Persistent queue",
                "Restore the play queue on startup.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.persistent_queue),
            ),
        ]
    }
}

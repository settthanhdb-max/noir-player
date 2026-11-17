//! Shared UI state types.

use noir_core::models::{MediaItem, MediaKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Player,
    Library,
    Playlists,
    History,
    Settings,
}

impl Screen {
    pub const ALL: [Screen; 6] = [
        Self::Home,
        Self::Player,
        Self::Library,
        Self::Playlists,
        Self::History,
        Self::Settings,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Home => "nav.home",
            Self::Player => "nav.player",
            Self::Library => "nav.library",
            Self::Playlists => "nav.playlists",
            Self::History => "nav.history",
            Self::Settings => "nav.settings",
        }
    }

    /// Index into [`noir_settings::schema::appearance::STARTUP_SCREEN_OPTIONS`].
    pub fn from_startup_index(index: usize) -> Self {
        match index {
            1 => Self::Player,
            2 => Self::Library,
            3 => Self::Playlists,
            4 => Self::History,
            _ => Self::Home,
        }
    }
}

/// Repeat behavior of the queue. Mirrors `PlaylistSettings::repeat_mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::One,
            2 => Self::All,
            _ => Self::Off,
        }
    }
}

/// One queued media file.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueEntry {
    pub path: String,
    pub title: String,
    pub kind: MediaKind,
    pub duration: Option<f64>,
}

impl QueueEntry {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path: String = path.into();
        let title = std::path::Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let ext = std::path::Path::new(&path)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        Self {
            path,
            title,
            kind: MediaKind::from_extension(&ext),
            duration: None,
        }
    }

    pub fn from_media(item: &MediaItem) -> Self {
        Self {
            path: item.path.to_string_lossy().to_string(),
            title: item.title.clone(),
            kind: item.kind,
            duration: item.duration,
        }
    }
}

/// What the settings screen wants from a native file dialog.
#[derive(Debug, Clone)]
pub enum PickRequest {
    Directory { key: String },
    File { key: String },
    ExportConfig,
    ImportConfig,
    ExportKeys,
    ImportKeys,
    ExportM3u { paths: Vec<String> },
    ImportM3u,
    OpenSubtitle,
    ScreenshotDir,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screens_are_complete() {
        assert_eq!(Screen::ALL.len(), 6);
        assert_eq!(Screen::from_startup_index(0), Screen::Home);
        assert_eq!(Screen::from_startup_index(4), Screen::History);
        assert_eq!(Screen::from_startup_index(99), Screen::Home);
    }

    #[test]
    fn repeat_mode_cycles_from_index() {
        assert_eq!(RepeatMode::from_index(0), RepeatMode::Off);
        assert_eq!(RepeatMode::from_index(1), RepeatMode::One);
        assert_eq!(RepeatMode::from_index(2), RepeatMode::All);
        assert_eq!(RepeatMode::from_index(7), RepeatMode::Off);
    }

    #[test]
    fn queue_entry_from_path_extracts_title_and_kind() {
        let e = QueueEntry::from_path("/media/Movies/Big Buck Bunny.mkv");
        assert_eq!(e.title, "Big Buck Bunny");
        assert_eq!(e.path, "/media/Movies/Big Buck Bunny.mkv");
        assert!(e.duration.is_none());
    }
}

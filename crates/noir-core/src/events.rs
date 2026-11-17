//! Event types exchanged between the playback engine, the library and the UI.
//!
//! The engine never calls the UI directly; it publishes [`PlaybackEvent`]s on a
//! channel and the UI drains them once per frame. UI-originated actions travel
//! the other way as commands (see `noir-playback::controller`).

use crate::models::{ChapterInfo, MediaStats, TrackInfo};
use crate::state::PlaybackState;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Events published by the playback engine.
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackEvent {
    StateChanged(PlaybackState),
    /// Current position and total duration in seconds.
    PositionChanged {
        position: f64,
        duration: f64,
    },
    MediaLoaded {
        title: String,
        duration: f64,
        tracks: Vec<TrackInfo>,
        chapters: Vec<ChapterInfo>,
        stats: MediaStats,
    },
    TracksChanged(Vec<TrackInfo>),
    StatsChanged(MediaStats),
    VolumeChanged {
        volume: u32,
        muted: bool,
    },
    SpeedChanged(f64),
    ChaptersChanged(Vec<ChapterInfo>),
    EndOfFile,
    FileLoaded {
        path: String,
    },
    Error(String),
    /// Backend reported a property useful for the OSD.
    OsdMessage(String),
}

/// Events produced by the library subsystem (scanner, database, history).
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryEvent {
    ScanStarted { directory: String },
    ScanProgress { scanned: u64, found: u64 },
    ScanFinished { directory: String, found: u64 },
    ScanFailed { directory: String, error: String },
    LibraryUpdated,
    HistoryUpdated,
}

/// Events emitted by the settings subsystem.
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsEvent {
    /// A setting changed; carries the dotted key path.
    Changed {
        key: String,
    },
    Loaded,
    Saved,
    Reset {
        scope: Option<String>,
    },
}

/// Aggregated engine event bus: everything the UI may react to.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    /// Boxed: `PlaybackEvent::MediaLoaded` is by far the largest variant.
    Playback(Box<PlaybackEvent>),
    Library(LibraryEvent),
    Settings(SettingsEvent),
}

impl From<PlaybackEvent> for EngineEvent {
    fn from(e: PlaybackEvent) -> Self {
        EngineEvent::Playback(Box::new(e))
    }
}

impl From<LibraryEvent> for EngineEvent {
    fn from(e: LibraryEvent) -> Self {
        EngineEvent::Library(e)
    }
}

impl From<SettingsEvent> for EngineEvent {
    fn from(e: SettingsEvent) -> Self {
        EngineEvent::Settings(e)
    }
}

/// A record of one playback session, used by history persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub path: String,
    pub title: String,
    pub last_played: SystemTime,
    /// Position where playback stopped, in seconds.
    pub position: f64,
    /// Total duration in seconds, when known.
    pub duration: Option<f64>,
    /// 0.0..=1.0 completion ratio at stop time.
    pub completion: f32,
}

/// UI-side event for cross-panel notifications (toasts, status bar).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiEvent {
    Info(String),
    Warning(String),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_conversion() {
        let e: EngineEvent = PlaybackEvent::EndOfFile.into();
        assert_eq!(e, EngineEvent::Playback(Box::new(PlaybackEvent::EndOfFile)));
        let e: EngineEvent = LibraryEvent::LibraryUpdated.into();
        assert!(matches!(e, EngineEvent::Library(_)));
    }

    #[test]
    fn history_entry_serializes() {
        let entry = HistoryEntry {
            path: "/a.mkv".into(),
            title: "a".into(),
            last_played: SystemTime::UNIX_EPOCH,
            position: 12.5,
            duration: Some(100.0),
            completion: 0.125,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
    }
}

//! Data models shared across subsystems.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Kind of media content, derived from the file extension / backend probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
    Image,
    #[default]
    Unknown,
}

impl MediaKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "mpg" | "mpeg" | "ts"
            | "m2ts" | "m4v" | "ogv" | "3gp" => Self::Video,
            "mp3" | "aac" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "alac" | "ac3" | "dts"
            | "wma" | "ape" | "wv" => Self::Audio,
            "jpg" | "jpeg" | "png" | "bmp" | "gif" | "webp" | "avif" => Self::Image,
            _ => Self::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Image => "Image",
            Self::Unknown => "Unknown",
        }
    }
}

/// A media entry known to the application (library row, playlist item, …).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Option<i64>,
    pub path: PathBuf,
    pub title: String,
    pub kind: MediaKind,
    /// Duration in seconds, when known.
    pub duration: Option<f64>,
    pub size_bytes: Option<u64>,
    pub added_at: Option<SystemTime>,
    pub last_played_at: Option<SystemTime>,
    pub play_count: u32,
    pub favorite: bool,
}

impl MediaItem {
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        let kind = path
            .extension()
            .and_then(|e| e.to_str())
            .map(MediaKind::from_extension)
            .unwrap_or(MediaKind::Unknown);
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        Self {
            id: None,
            path,
            title,
            kind,
            duration: None,
            size_bytes: None,
            added_at: None,
            last_played_at: None,
            play_count: 0,
            favorite: false,
        }
    }

    pub fn extension(&self) -> String {
        self.path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_uppercase()
    }
}

/// A selectable stream inside the currently loaded media.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackInfo {
    /// Backend-native track id (mpv track list id).
    pub id: u64,
    pub kind: TrackKind,
    pub title: Option<String>,
    pub lang: Option<String>,
    /// True when this track is the currently selected one.
    pub selected: bool,
    pub default: bool,
    pub forced: bool,
    /// Codec name reported by the backend, e.g. "h264", "aac".
    pub codec: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackKind {
    Video,
    Audio,
    Subtitle,
}

impl TrackKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Subtitle => "Subtitle",
        }
    }
}

/// Chapter marker inside the current media.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterInfo {
    pub index: usize,
    pub title: String,
    /// Start time in seconds.
    pub start: f64,
}

/// Live technical statistics of the playback engine (for the info overlay).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MediaStats {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub fps: Option<f64>,
    pub video_bitrate: Option<f64>,
    pub audio_bitrate: Option<f64>,
    pub hw_decoder: Option<String>,
    pub dropped_frames: Option<u64>,
    pub output_fps: Option<f64>,
    pub av_sync: Option<f64>,
    pub cache_used: Option<f64>,
    pub buffering_state: Option<f64>,
}

impl MediaStats {
    /// One-line summary, e.g. `1920x1080 h264 23.976fps`.
    pub fn video_summary(&self) -> Option<String> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => {
                let fps = self.fps.map(|f| format!(" {f:.3}fps")).unwrap_or_default();
                let codec = self
                    .video_codec
                    .as_deref()
                    .map(|c| format!(" {c}"))
                    .unwrap_or_default();
                Some(format!("{w}x{h}{codec}{fps}"))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_from_extension() {
        assert_eq!(MediaKind::from_extension("MKV"), MediaKind::Video);
        assert_eq!(MediaKind::from_extension("flac"), MediaKind::Audio);
        assert_eq!(MediaKind::from_extension("png"), MediaKind::Image);
        assert_eq!(MediaKind::from_extension("txt"), MediaKind::Unknown);
    }

    #[test]
    fn media_item_from_path() {
        let item = MediaItem::from_path("/media/Blade Runner 2049.mkv");
        assert_eq!(item.title, "Blade Runner 2049");
        assert_eq!(item.kind, MediaKind::Video);
        assert_eq!(item.extension(), "MKV");
    }

    #[test]
    fn stats_summary() {
        let stats = MediaStats {
            width: Some(1920),
            height: Some(1080),
            video_codec: Some("h264".into()),
            fps: Some(23.976),
            ..Default::default()
        };
        assert_eq!(stats.video_summary().unwrap(), "1920x1080 h264 23.976fps");
    }
}

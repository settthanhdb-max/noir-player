//! The engine contract every playback backend must implement.

use crate::Capabilities;
use crossbeam_channel::{Receiver, Sender};
use noir_core::events::PlaybackEvent;
use noir_core::models::{ChapterInfo, TrackInfo};
use noir_core::state::PlaybackState;
use noir_core::NoirError;
use std::path::Path;
use std::time::Duration;

/// Commands the UI can send to the engine. Commands are cheap to clone and
/// travel over a channel, so the UI thread never blocks on the backend.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineCommand {
    Open {
        path: String,
        start_position: Option<f64>,
    },
    Play,
    Pause,
    TogglePlay,
    Stop,
    SeekAbsolute {
        position: f64,
    },
    SeekRelative {
        offset: f64,
    },
    FrameStep {
        direction: i8,
    },
    SetSpeed {
        speed: f64,
    },
    SetVolume {
        volume: u32,
    },
    SetMuted {
        muted: bool,
    },
    SelectTrack {
        kind: noir_core::models::TrackKind,
        id: u64,
    },
    SetSubtitleDelay {
        seconds: f64,
    },
    SetAudioDelay {
        seconds: f64,
    },
    SetVideoEqualizer {
        brightness: i32,
        contrast: i32,
        saturation: i32,
        gamma: i32,
        hue: i32,
    },
    SetAspect {
        aspect: Option<String>,
    },
    SetZoomPan {
        zoom: f64,
        pan_x: f64,
        pan_y: f64,
    },
    SetPanScan {
        value: f64,
    },
    SetRotation {
        degrees: u32,
    },
    SetFlip {
        horizontal: bool,
        vertical: bool,
    },
    SetDeinterlace {
        mode: String,
    },
    SetHwdec {
        mode: String,
    },
    SetAudioFilters {
        chain: Option<String>,
    },
    SetSubDelay {
        seconds: f64,
    },
    SetSubVisibility {
        visible: bool,
    },
    SetAbLoop {
        a: Option<f64>,
        b: Option<f64>,
    },
    Screenshot {
        path: Option<String>,
    },
    LoadSubtitle {
        path: String,
    },
    ApplyRawOptions {
        options: Vec<(String, String)>,
    },
    Shutdown,
}

/// A point-in-time snapshot of engine state for synchronous UI reads.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngineSnapshot {
    pub state: PlaybackState,
    pub position: f64,
    pub duration: f64,
    pub speed: f64,
    pub volume: u32,
    pub muted: bool,
    pub tracks: Vec<TrackInfo>,
    pub chapters: Vec<ChapterInfo>,
    pub stats: noir_core::models::MediaStats,
    pub current_path: Option<String>,
}

/// The playback backend contract. All methods are non-blocking: engines run
/// their own worker and report through the event channel.
pub trait PlaybackEngine: Send + Sync {
    /// Backend name and version for the capability report.
    fn name(&self) -> String;

    fn capabilities(&self) -> Capabilities;

    /// Subscribe to engine events. Multiple subscribers are allowed.
    fn subscribe(&self) -> Receiver<PlaybackEvent>;

    /// Queue a command. Returns immediately.
    fn send(&self, command: EngineCommand);

    /// Current state snapshot (lock-free where possible, never blocks long).
    fn snapshot(&self) -> EngineSnapshot;

    /// Attach the video output to a native window handle (HWND on Windows).
    /// Returns an error when the backend cannot render into this surface.
    fn attach_window(&self, _window: *mut std::ffi::c_void) -> Result<(), NoirError> {
        Err(NoirError::Renderer(
            "this backend does not support native window embedding".into(),
        ))
    }

    /// Detach the video output.
    fn detach_window(&self) {}

    /// Register a callback invoked when the video output must be resized.
    fn set_resize_handler(&self, _sender: Sender<(u32, u32)>) {}

    /// Block until shutdown, running the engine's internal loop.
    fn run(&self);

    /// Request cooperative shutdown of [`PlaybackEngine::run`].
    fn request_shutdown(&self);

    /// Probe a media file without playing it. Used by the library scanner.
    fn probe(&self, path: &Path) -> Result<noir_core::models::MediaStats, NoirError> {
        let _ = path;
        Err(NoirError::UnsupportedMedia {
            reason: "probing not supported by this backend".into(),
        })
    }

    /// Recommended poll interval for the UI when draining events. Engines
    /// that push events can return a long interval.
    fn event_poll_hint(&self) -> Duration {
        Duration::from_millis(100)
    }
}

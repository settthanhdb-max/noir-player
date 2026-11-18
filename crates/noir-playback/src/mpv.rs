//! libmpv backend.
//!
//! Integration strategy: mpv renders into a native child window via the
//! `wid` property (HWND on Windows). This is the most reliable path on
//! Windows — mpv owns its D3D11 swapchain and all GPU synchronization, so
//! there is no shared GL context and no frame-copy. The egui UI docks the
//! control panel next to/below the video surface and resizes the child
//! window when overlays collapse; transient OSD messages use mpv's own OSD.
//!
//! The render-API (OpenGL texture) path is intentionally not used: libmpv2's
//! `RenderContext` borrows the `Mpv` handle, which cannot be stored
//! alongside it without unsafe lifetime surgery, and the workspace denies
//! unsafe code.

use crate::capabilities::Capabilities;
use crate::engine::{EngineCommand, EngineSnapshot, PlaybackEngine};
use crossbeam_channel::{unbounded, Receiver, Sender};
use libmpv2::events::{Event, PropertyData};
use libmpv2::{Format, Mpv};
use libmpv2_sys::{
    mpv_end_file_reason_MPV_END_FILE_REASON_EOF, mpv_end_file_reason_MPV_END_FILE_REASON_ERROR,
};
use noir_core::events::PlaybackEvent;
use noir_core::models::{ChapterInfo, MediaStats, TrackInfo, TrackKind};
use noir_core::state::PlaybackState;
use noir_core::NoirError;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, trace, warn};

pub struct MpvEngine {
    mpv: Arc<Mutex<Mpv>>,
    event_tx: Mutex<Vec<Sender<PlaybackEvent>>>,
    shutdown: AtomicBool,
    last_position: Mutex<f64>,
    last_open_path: Mutex<String>,
    recent_errors: Mutex<VecDeque<String>>,
}

impl MpvEngine {
    /// Create an mpv instance. Returns `None` when native libmpv cannot be
    /// loaded, so the caller can fall back to the mock engine.
    pub fn try_new() -> Option<Self> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_option("config", false)?;
            init.set_option("input-default-bindings", false)?;
            init.set_option("input-vo-keyboard", false)?;
            init.set_option("osc", false)?;
            init.set_option("idle", true)?;
            init.set_option("keep-open", true)?;
            init.set_option("audio-display", "no")
        })
        .ok()?;
        let mpv = Arc::new(Mutex::new(mpv));
        {
            let mpv = mpv.lock();
            observe_core_properties(&mpv);
            let _ = mpv.command("request-log-messages", &["warn"]);
        }
        Some(Self {
            mpv,
            event_tx: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
            last_position: Mutex::new(0.0),
            last_open_path: Mutex::new(String::new()),
            recent_errors: Mutex::new(VecDeque::new()),
        })
    }

    fn publish(&self, event: PlaybackEvent) {
        self.event_tx
            .lock()
            .retain(|tx| tx.send(event.clone()).is_ok());
    }

    fn set_or_warn(&self, prop: &str, value: impl libmpv2::SetData) {
        let mpv = self.mpv.lock();
        if let Err(e) = mpv.set_property(prop, value) {
            warn!("mpv: failed to set {prop}: {e}");
        }
    }

    fn command_or_warn(&self, name: &str, args: &[&str]) {
        let mpv = self.mpv.lock();
        if let Err(e) = mpv.command(name, args) {
            warn!("mpv: command {name} failed: {e}");
            self.publish(PlaybackEvent::Error(format!("{name}: {e}")));
        }
    }

    fn get_f64(&self, prop: &str) -> Option<f64> {
        let mpv = self.mpv.lock();
        mpv.get_property::<f64>(prop).ok()
    }

    fn get_string(&self, prop: &str) -> Option<String> {
        let mpv = self.mpv.lock();
        mpv.get_property::<String>(prop).ok()
    }

    fn get_bool(&self, prop: &str) -> Option<bool> {
        let mpv = self.mpv.lock();
        mpv.get_property::<bool>(prop).ok()
    }

    /// Event loop translating mpv events into [`PlaybackEvent`]s.
    fn event_loop(&self) {
        let mut last_publish = std::time::Instant::now();
        loop {
            if self.shutdown.load(Ordering::Relaxed) {
                break;
            }
            let pending: Vec<PlaybackEvent> = {
                let mpv = self.mpv.lock();
                let mut pending = Vec::new();
                while let Some(ev) = mpv.wait_event(0.0) {
                    match ev {
                        Ok(Event::StartFile) => {
                            pending.push(PlaybackEvent::StateChanged(PlaybackState::Loading));
                        }
                        Ok(Event::FileLoaded) => {
                            let path = get_str(&mpv, "path").unwrap_or_default();
                            let duration = get_f64(&mpv, "duration").unwrap_or(0.0);
                            pending.push(PlaybackEvent::FileLoaded { path: path.clone() });
                            pending.push(PlaybackEvent::MediaLoaded {
                                title: std::path::Path::new(&path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("Unknown")
                                    .to_string(),
                                duration,
                                tracks: read_tracks(&mpv),
                                chapters: read_chapters(&mpv),
                                stats: read_stats(&mpv),
                            });
                            pending.push(PlaybackEvent::StateChanged(PlaybackState::Playing));
                        }
                        Ok(Event::LogMessage {
                            prefix,
                            level,
                            text,
                            ..
                        }) => {
                            if matches!(level, "error" | "fatal") {
                                warn!("mpv {prefix}: {text}");
                                let mut errors = self.recent_errors.lock();
                                errors.push_back(text.trim().to_string());
                                if errors.len() > 8 {
                                    errors.pop_front();
                                }
                            } else {
                                debug!("mpv {prefix}: {text}");
                            }
                        }
                        Ok(Event::EndFile(reason)) => {
                            if reason == mpv_end_file_reason_MPV_END_FILE_REASON_EOF {
                                pending.push(PlaybackEvent::EndOfFile);
                                pending.push(PlaybackEvent::StateChanged(PlaybackState::Ended));
                            } else if reason == mpv_end_file_reason_MPV_END_FILE_REASON_ERROR {
                                let path = self.last_open_path.lock().clone();
                                let detail = self
                                    .recent_errors
                                    .lock()
                                    .pop_back()
                                    .unwrap_or_else(|| "unknown mpv error".into());
                                let message = if path.is_empty() {
                                    format!("playback failed: {detail}")
                                } else {
                                    format!("cannot open {path}: {detail}")
                                };
                                warn!("mpv play failed: {message}");
                                pending.push(PlaybackEvent::Error(message.clone()));
                                pending.push(PlaybackEvent::StateChanged(PlaybackState::Error(
                                    message,
                                )));
                            } else {
                                debug!("mpv EndFile reason={reason}");
                                pending.push(PlaybackEvent::StateChanged(PlaybackState::Idle));
                            }
                        }
                        Ok(Event::PlaybackRestart) => {
                            pending.push(PlaybackEvent::StateChanged(derive_state(&mpv)));
                        }
                        Ok(Event::Shutdown) => {
                            pending.push(PlaybackEvent::StateChanged(PlaybackState::Idle));
                            self.shutdown.store(true, Ordering::Relaxed);
                        }
                        Ok(Event::PropertyChange { name, change, .. }) => {
                            handle_property_change(&mpv, &mut pending, name, &change);
                        }
                        Ok(_) => {}
                        Err(e) => warn!("mpv event error: {e}"),
                    }
                    if pending.len() >= 256 {
                        break;
                    }
                }
                pending
            };
            for event in pending {
                self.publish(event);
            }
            // Publish position at most every 100 ms; the timeline needs a
            // steady stream but not per-frame spam.
            if last_publish.elapsed() >= Duration::from_millis(100) {
                last_publish = std::time::Instant::now();
                if let Some(position) = self.get_f64("time-pos") {
                    let mut last = self.last_position.lock();
                    if (position - *last).abs() > 0.01 {
                        *last = position;
                        drop(last);
                        let duration = self.get_f64("duration").unwrap_or(0.0);
                        self.publish(PlaybackEvent::PositionChanged { position, duration });
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        trace!("mpv event loop exited");
    }
}

fn observe_core_properties(mpv: &Mpv) {
    let _ = mpv.observe_property("pause", Format::Flag, 1);
    let _ = mpv.observe_property("speed", Format::Double, 2);
    let _ = mpv.observe_property("volume", Format::Int64, 3);
    let _ = mpv.observe_property("mute", Format::Flag, 4);
    let _ = mpv.observe_property("eof-reached", Format::Flag, 5);
    let _ = mpv.observe_property("seeking", Format::Flag, 6);
    let _ = mpv.observe_property("hwdec-current", Format::String, 7);
    let _ = mpv.observe_property("dropped-frames", Format::Int64, 8);
}

fn get_f64(mpv: &Mpv, prop: &str) -> Option<f64> {
    mpv.get_property::<f64>(prop).ok()
}

fn get_i64(mpv: &Mpv, prop: &str) -> Option<i64> {
    mpv.get_property::<i64>(prop).ok()
}

fn get_str(mpv: &Mpv, prop: &str) -> Option<String> {
    mpv.get_property::<String>(prop).ok()
}

fn get_bool(mpv: &Mpv, prop: &str) -> Option<bool> {
    mpv.get_property::<bool>(prop).ok()
}

fn read_tracks(mpv: &Mpv) -> Vec<TrackInfo> {
    let count = get_i64(mpv, "track-list/count").unwrap_or(0);
    let mut tracks = Vec::with_capacity(count as usize);
    for i in 0..count {
        let base = format!("track-list/{i}");
        let kind = match get_str(mpv, &format!("{base}/type")).as_deref() {
            Some("video") => TrackKind::Video,
            Some("audio") => TrackKind::Audio,
            Some("sub") => TrackKind::Subtitle,
            _ => continue,
        };
        tracks.push(TrackInfo {
            id: get_i64(mpv, &format!("{base}/id")).unwrap_or(0) as u64,
            kind,
            title: get_str(mpv, &format!("{base}/title")),
            lang: get_str(mpv, &format!("{base}/lang")),
            selected: get_bool(mpv, &format!("{base}/selected")).unwrap_or(false),
            default: get_bool(mpv, &format!("{base}/default")).unwrap_or(false),
            forced: get_bool(mpv, &format!("{base}/forced")).unwrap_or(false),
            codec: get_str(mpv, &format!("{base}/codec")),
        });
    }
    tracks
}

fn read_chapters(mpv: &Mpv) -> Vec<ChapterInfo> {
    let count = get_i64(mpv, "chapter-list/count").unwrap_or(0);
    let mut chapters = Vec::with_capacity(count as usize);
    for i in 0..count {
        chapters.push(ChapterInfo {
            index: i as usize,
            title: get_str(mpv, &format!("chapter-list/{i}/title"))
                .unwrap_or_else(|| format!("Chapter {}", i + 1)),
            start: get_f64(mpv, &format!("chapter-list/{i}/time")).unwrap_or(0.0),
        });
    }
    chapters
}

fn read_stats(mpv: &Mpv) -> MediaStats {
    MediaStats {
        width: get_i64(mpv, "video-params/w").map(|v| v as u32),
        height: get_i64(mpv, "video-params/h").map(|v| v as u32),
        video_codec: get_str(mpv, "video-codec"),
        audio_codec: get_str(mpv, "audio-codec-name"),
        fps: get_f64(mpv, "container-fps"),
        video_bitrate: get_f64(mpv, "video-bitrate"),
        audio_bitrate: get_f64(mpv, "audio-bitrate"),
        hw_decoder: get_str(mpv, "hwdec-current").filter(|s| s != "no"),
        dropped_frames: get_i64(mpv, "dropped-frames").map(|v| v as u64),
        output_fps: get_f64(mpv, "estimated-vf-fps"),
        av_sync: get_f64(mpv, "avsync"),
        cache_used: get_f64(mpv, "cache-buffering-state").map(|v| v / 100.0),
        buffering_state: get_f64(mpv, "cache-buffering-state").map(|v| v / 100.0),
    }
}

fn derive_state(mpv: &Mpv) -> PlaybackState {
    if get_str(mpv, "path").is_none() {
        return PlaybackState::Idle;
    }
    let buffering = get_f64(mpv, "cache-buffering-state")
        .map(|v| v > 0.0 && v < 100.0)
        .unwrap_or(false);
    if get_bool(mpv, "seeking").unwrap_or(false) || buffering {
        return PlaybackState::Buffering;
    }
    if get_bool(mpv, "pause").unwrap_or(false) {
        return PlaybackState::Paused;
    }
    if get_bool(mpv, "eof-reached").unwrap_or(false) {
        return PlaybackState::Ended;
    }
    PlaybackState::Playing
}

fn handle_property_change(
    mpv: &Mpv,
    pending: &mut Vec<PlaybackEvent>,
    name: &str,
    change: &PropertyData<'_>,
) {
    match name {
        "pause" => pending.push(PlaybackEvent::StateChanged(derive_state(mpv))),
        "speed" => {
            if let PropertyData::Double(speed) = change {
                pending.push(PlaybackEvent::SpeedChanged(*speed));
            }
        }
        "volume" | "mute" => {
            let volume = get_i64(mpv, "volume").unwrap_or(100) as u32;
            let muted = get_bool(mpv, "mute").unwrap_or(false);
            pending.push(PlaybackEvent::VolumeChanged { volume, muted });
        }
        "track-list" => pending.push(PlaybackEvent::TracksChanged(read_tracks(mpv))),
        "chapter-list" => pending.push(PlaybackEvent::ChaptersChanged(read_chapters(mpv))),
        _ => {}
    }
}

impl PlaybackEngine for MpvEngine {
    fn name(&self) -> String {
        self.get_string("mpv-version")
            .unwrap_or_else(|| "libmpv".into())
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::mpv(&self.get_string("mpv-version").unwrap_or_default())
    }

    fn subscribe(&self) -> Receiver<PlaybackEvent> {
        let (tx, rx) = unbounded();
        self.event_tx.lock().push(tx);
        rx
    }

    fn send(&self, command: EngineCommand) {
        use EngineCommand::*;
        match command {
            Open {
                path,
                start_position,
            } => {
                *self.last_open_path.lock() = path.clone();
                self.recent_errors.lock().clear();
                let pos_arg = start_position
                    .filter(|p| *p > 0.5)
                    .map(|p| format!("start={p}"))
                    .unwrap_or_default();
                let mpv = self.mpv.lock();
                let result = if pos_arg.is_empty() {
                    mpv.command("loadfile", &[&path, "replace"])
                } else {
                    // Since mpv 0.38.0 the options list is the 4th argument;
                    // the 3rd (insertion index) must be -1 when the 4th is
                    // used, otherwise the command fails with an invalid
                    // parameter error.
                    mpv.command("loadfile", &[&path, "replace", "-1", &pos_arg])
                };
                if let Err(e) = result {
                    warn!("mpv: failed to open {path}: {e}");
                    self.publish(PlaybackEvent::Error(format!("cannot open {path}: {e}")));
                    self.publish(PlaybackEvent::StateChanged(PlaybackState::Error(format!(
                        "cannot open {path}"
                    ))));
                } else {
                    let _ = mpv.set_property("pause", false);
                    drop(mpv);
                    self.publish(PlaybackEvent::StateChanged(PlaybackState::Loading));
                }
            }
            Play => self.set_or_warn("pause", false),
            Pause => self.set_or_warn("pause", true),
            TogglePlay => {
                let paused = self.get_bool("pause").unwrap_or(false);
                self.set_or_warn("pause", !paused);
            }
            Stop => self.command_or_warn("stop", &[]),
            SeekAbsolute { position } => self.set_or_warn("time-pos", position),
            SeekRelative { offset } => {
                self.command_or_warn("seek", &[&format!("{offset:.3}"), "relative"])
            }
            FrameStep { direction } => {
                if direction > 0 {
                    self.command_or_warn("frame-step", &[]);
                } else {
                    self.command_or_warn("frame-back-step", &[]);
                }
            }
            SetSpeed { speed } => self.set_or_warn("speed", speed),
            SetVolume { volume } => self.set_or_warn("volume", volume as i64),
            SetMuted { muted } => self.set_or_warn("mute", muted),
            SelectTrack { kind, id } => {
                let prop = match kind {
                    TrackKind::Video => "vid",
                    TrackKind::Audio => "aid",
                    TrackKind::Subtitle => "sid",
                };
                self.set_or_warn(prop, id as i64);
            }
            SetSubtitleDelay { seconds } => self.set_or_warn("sub-delay", seconds),
            SetAudioDelay { seconds } => self.set_or_warn("audio-delay", seconds),
            SetVideoEqualizer {
                brightness,
                contrast,
                saturation,
                gamma,
                hue,
            } => {
                self.set_or_warn("brightness", brightness as i64);
                self.set_or_warn("contrast", contrast as i64);
                self.set_or_warn("saturation", saturation as i64);
                self.set_or_warn("gamma", gamma as i64);
                self.set_or_warn("hue", hue as i64);
            }
            SetAspect { aspect } => {
                let value = aspect.unwrap_or_else(|| "no".into());
                self.set_or_warn("video-aspect-override", value);
            }
            SetZoomPan { zoom, pan_x, pan_y } => {
                // mpv's video-zoom is logarithmic: factor 2.0 => 1.0.
                let log_zoom = if zoom > 0.0 { zoom.log2() } else { 0.0 };
                self.set_or_warn("video-zoom", log_zoom);
                self.set_or_warn("video-pan-x", pan_x);
                self.set_or_warn("video-pan-y", pan_y);
            }
            SetRotation { degrees } => self.set_or_warn("video-rotate", degrees.to_string()),
            SetFlip {
                horizontal,
                vertical,
            } => {
                self.set_or_warn("hflip", horizontal);
                self.set_or_warn("vflip", vertical);
            }
            SetDeinterlace { mode } => self.set_or_warn("deinterlace", mode),
            SetHwdec { mode } => self.set_or_warn("hwdec", mode),
            SetAudioFilters { chain } => {
                let value = chain.unwrap_or_default();
                self.set_or_warn("af", value);
            }
            SetSubDelay { seconds } => self.set_or_warn("sub-delay", seconds),
            SetSubVisibility { visible } => self.set_or_warn("sub-visibility", visible),
            SetAbLoop { a, b } => {
                match a {
                    Some(t) => self.set_or_warn("ab-loop-a", t),
                    None => self.set_or_warn("ab-loop-a", "no"),
                }
                match b {
                    Some(t) => self.set_or_warn("ab-loop-b", t),
                    None => self.set_or_warn("ab-loop-b", "no"),
                }
            }
            Screenshot { path } => match path {
                Some(p) => self.command_or_warn("screenshot-to-file", &[&p]),
                None => self.command_or_warn("screenshot", &[]),
            },
            LoadSubtitle { path } => self.command_or_warn("sub-add", &[&path, "auto"]),
            ApplyRawOptions { options } => {
                let mpv = self.mpv.lock();
                for (key, value) in options {
                    if let Err(e) = mpv.set_property(&key, value.clone()) {
                        warn!("mpv: custom option {key}={value} rejected: {e}");
                        self.publish(PlaybackEvent::OsdMessage(format!(
                            "option {key} rejected: {e}"
                        )));
                    }
                }
            }
            Shutdown => self.request_shutdown(),
        }
    }

    fn snapshot(&self) -> EngineSnapshot {
        let mpv = self.mpv.lock();
        EngineSnapshot {
            state: derive_state(&mpv),
            position: get_f64(&mpv, "time-pos").unwrap_or(0.0),
            duration: get_f64(&mpv, "duration").unwrap_or(0.0),
            speed: get_f64(&mpv, "speed").unwrap_or(1.0),
            volume: get_i64(&mpv, "volume").unwrap_or(100) as u32,
            muted: get_bool(&mpv, "mute").unwrap_or(false),
            tracks: read_tracks(&mpv),
            chapters: read_chapters(&mpv),
            stats: read_stats(&mpv),
            current_path: get_str(&mpv, "path"),
        }
    }

    fn attach_window(&self, window: *mut std::ffi::c_void) -> Result<(), NoirError> {
        let hwnd = window as i64;
        let mpv = self.mpv.lock();
        mpv.set_property("wid", hwnd)
            .map_err(|e| NoirError::Renderer(format!("wid attach failed: {e}")))
    }

    fn detach_window(&self) {
        let mpv = self.mpv.lock();
        let _ = mpv.set_property("wid", 0i64);
    }

    fn run(&self) {
        self.event_loop();
    }

    fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    fn probe(&self, path: &std::path::Path) -> Result<MediaStats, NoirError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| NoirError::UnsupportedMedia {
                reason: "non-UTF8 path".into(),
            })?
            .to_string();
        let client = {
            let mpv = self.mpv.lock();
            mpv.create_client(Some("noir-probe"))
                .map_err(|e| NoirError::Engine(format!("probe client: {e}")))?
        };
        let _ = client.set_property("vo", "null");
        let _ = client.set_property("ao", "null");
        let _ = client.set_property("pause", true);
        client
            .command("loadfile", &[&path_str, "replace"])
            .map_err(|e| NoirError::Engine(format!("probe loadfile: {e}")))?;
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            if std::time::Instant::now() > deadline {
                return Err(NoirError::Engine("probe timed out".into()));
            }
            if let Some(Ok(Event::FileLoaded)) = client.wait_event(0.1) {
                break;
            }
        }
        let stats = MediaStats {
            width: client
                .get_property::<i64>("video-params/w")
                .ok()
                .map(|v| v as u32),
            height: client
                .get_property::<i64>("video-params/h")
                .ok()
                .map(|v| v as u32),
            video_codec: client.get_property::<String>("video-codec").ok(),
            audio_codec: client.get_property::<String>("audio-codec-name").ok(),
            fps: client.get_property::<f64>("container-fps").ok(),
            video_bitrate: client.get_property::<f64>("video-bitrate").ok(),
            audio_bitrate: client.get_property::<f64>("audio-bitrate").ok(),
            hw_decoder: None,
            dropped_frames: None,
            output_fps: None,
            av_sync: None,
            cache_used: None,
            buffering_state: None,
        };
        let _ = client.command("stop", &[]);
        Ok(stats)
    }

    fn event_poll_hint(&self) -> Duration {
        Duration::from_millis(50)
    }
}

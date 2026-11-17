//! Mock engine: full command/event plumbing with no media output.
//!
//! Used in tests and as an honest fallback when the native backend is
//! unavailable. It advances a virtual clock while "playing" so the UI can be
//! exercised end-to-end, and it reports `Capabilities::mock()` so the UI can
//! state the limitation instead of faking playback.

use crate::capabilities::Capabilities;
use crate::engine::{EngineCommand, EngineSnapshot, PlaybackEngine};
use crossbeam_channel::{unbounded, Receiver, Sender};
use noir_core::events::PlaybackEvent;
use noir_core::models::MediaStats;
use noir_core::state::PlaybackState;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct MockState {
    state: PlaybackState,
    position: f64,
    duration: f64,
    speed: f64,
    volume: u32,
    muted: bool,
    current_path: Option<String>,
    stats: MediaStats,
    /// Wall-clock instant when playback last (re)started.
    playing_since: Option<Instant>,
}

#[derive(Debug)]
pub struct MockEngine {
    state: Arc<Mutex<MockState>>,
    event_tx: Mutex<Vec<Sender<PlaybackEvent>>>,
    shutdown: AtomicBool,
}

impl MockEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                state: PlaybackState::Idle,
                position: 0.0,
                duration: 0.0,
                speed: 1.0,
                volume: 100,
                muted: false,
                current_path: None,
                stats: MediaStats::default(),
                playing_since: None,
            })),
            event_tx: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
        }
    }

    fn publish(&self, event: PlaybackEvent) {
        self.event_tx
            .lock()
            .retain(|tx| tx.send(event.clone()).is_ok());
    }

    fn advance_clock(&self, state: &mut MockState) {
        if let Some(since) = state.playing_since {
            state.position += since.elapsed().as_secs_f64() * state.speed;
            state.playing_since = Some(Instant::now());
            if state.duration > 0.0 && state.position >= state.duration {
                state.position = state.duration;
                state.state = PlaybackState::Ended;
                state.playing_since = None;
                self.publish(PlaybackEvent::EndOfFile);
                self.publish(PlaybackEvent::StateChanged(PlaybackState::Ended));
            }
        }
    }
}

impl Default for MockEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackEngine for MockEngine {
    fn name(&self) -> String {
        "mock".into()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities::mock()
    }

    fn subscribe(&self) -> Receiver<PlaybackEvent> {
        let (tx, rx) = unbounded();
        self.event_tx.lock().push(tx);
        rx
    }

    fn send(&self, command: EngineCommand) {
        let mut state = self.state.lock();
        self.advance_clock(&mut state);
        match command {
            EngineCommand::Open {
                path,
                start_position,
            } => {
                state.current_path = Some(path.clone());
                state.position = start_position.unwrap_or(0.0);
                state.duration = 600.0;
                state.state = PlaybackState::Playing;
                state.playing_since = Some(Instant::now());
                state.stats = MediaStats {
                    width: Some(1920),
                    height: Some(1080),
                    video_codec: Some("mock-video".into()),
                    audio_codec: Some("mock-audio".into()),
                    fps: Some(24.0),
                    ..Default::default()
                };
                self.publish(PlaybackEvent::FileLoaded { path });
                self.publish(PlaybackEvent::StateChanged(PlaybackState::Playing));
            }
            EngineCommand::Play | EngineCommand::TogglePlay => match state.state {
                PlaybackState::Playing => {
                    state.state = PlaybackState::Paused;
                    state.playing_since = None;
                    self.publish(PlaybackEvent::StateChanged(PlaybackState::Paused));
                }
                PlaybackState::Paused | PlaybackState::Ended | PlaybackState::Idle => {
                    state.state = PlaybackState::Playing;
                    state.playing_since = Some(Instant::now());
                    self.publish(PlaybackEvent::StateChanged(PlaybackState::Playing));
                }
                _ => {}
            },
            EngineCommand::Pause => {
                state.state = PlaybackState::Paused;
                state.playing_since = None;
                self.publish(PlaybackEvent::StateChanged(PlaybackState::Paused));
            }
            EngineCommand::Stop => {
                state.state = PlaybackState::Idle;
                state.position = 0.0;
                state.playing_since = None;
                self.publish(PlaybackEvent::StateChanged(PlaybackState::Idle));
            }
            EngineCommand::SeekAbsolute { position } => {
                state.position = position.clamp(0.0, state.duration.max(0.0));
                self.publish(PlaybackEvent::PositionChanged {
                    position: state.position,
                    duration: state.duration,
                });
            }
            EngineCommand::SeekRelative { offset } => {
                state.position = (state.position + offset).clamp(0.0, state.duration.max(0.0));
                self.publish(PlaybackEvent::PositionChanged {
                    position: state.position,
                    duration: state.duration,
                });
            }
            EngineCommand::SetSpeed { speed } => {
                self.advance_clock(&mut state);
                state.speed = speed;
                self.publish(PlaybackEvent::SpeedChanged(speed));
            }
            EngineCommand::SetVolume { volume } => {
                state.volume = volume;
                self.publish(PlaybackEvent::VolumeChanged {
                    volume,
                    muted: state.muted,
                });
            }
            EngineCommand::SetMuted { muted } => {
                state.muted = muted;
                self.publish(PlaybackEvent::VolumeChanged {
                    volume: state.volume,
                    muted,
                });
            }
            EngineCommand::Shutdown => {
                self.shutdown.store(true, Ordering::Relaxed);
            }
            _ => {
                // The mock honors the remaining commands as no-ops but never
                // pretends they had a real effect.
                self.publish(PlaybackEvent::OsdMessage(
                    "mock engine: command accepted without media output".into(),
                ));
            }
        }
    }

    fn snapshot(&self) -> EngineSnapshot {
        let mut state = self.state.lock();
        self.advance_clock(&mut state);
        EngineSnapshot {
            state: state.state.clone(),
            position: state.position,
            duration: state.duration,
            speed: state.speed,
            volume: state.volume,
            muted: state.muted,
            tracks: Vec::new(),
            chapters: Vec::new(),
            stats: state.stats.clone(),
            current_path: state.current_path.clone(),
        }
    }

    fn run(&self) {
        while !self.shutdown.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn request_shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    fn event_poll_hint(&self) -> Duration {
        Duration::from_millis(50)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noir_core::models::TrackKind as TK;

    #[test]
    fn open_play_pause_cycle() {
        let engine = MockEngine::new();
        let rx = engine.subscribe();
        engine.send(EngineCommand::Open {
            path: "x.mkv".into(),
            start_position: None,
        });
        assert_eq!(engine.snapshot().state, PlaybackState::Playing);
        engine.send(EngineCommand::TogglePlay);
        assert!(engine.snapshot().state.is_paused());
        engine.send(EngineCommand::TogglePlay);
        assert!(engine.snapshot().state.is_playing());
        let events: Vec<PlaybackEvent> = rx.try_iter().collect();
        assert!(events.contains(&PlaybackEvent::StateChanged(PlaybackState::Playing)));
        assert!(events.contains(&PlaybackEvent::StateChanged(PlaybackState::Paused)));
    }

    #[test]
    fn seek_clamps_to_duration() {
        let engine = MockEngine::new();
        engine.send(EngineCommand::Open {
            path: "x.mkv".into(),
            start_position: None,
        });
        engine.send(EngineCommand::SeekAbsolute { position: 5000.0 });
        assert!((engine.snapshot().position - 600.0).abs() < 0.1);
    }

    #[test]
    fn speed_affects_virtual_clock() {
        let engine = MockEngine::new();
        engine.send(EngineCommand::Open {
            path: "x.mkv".into(),
            start_position: None,
        });
        engine.send(EngineCommand::SetSpeed { speed: 2.0 });
        std::thread::sleep(Duration::from_millis(120));
        let pos = engine.snapshot().position;
        assert!(pos > 0.2, "position should have advanced at 2x, got {pos}");
    }

    #[test]
    fn capabilities_are_honest() {
        let engine = MockEngine::new();
        assert!(!engine.capabilities().video);
        assert_eq!(engine.name(), "mock");
        let _ = TK::Video;
    }
}

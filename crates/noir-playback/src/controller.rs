//! Controller: the single object the UI owns to talk to the engine.
//!
//! Commands flow UI -> controller -> engine worker thread.
//! Events flow engine -> broadcast channel -> UI (drained once per frame).

use crate::engine::{EngineCommand, EngineSnapshot, PlaybackEngine};
use crossbeam_channel::{unbounded, Receiver, Sender};
use noir_core::events::PlaybackEvent;
use noir_core::state::PlaybackState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::warn;

/// Thread-safe handle to a running engine.
#[derive(Clone)]
pub struct PlaybackController {
    engine: Arc<dyn PlaybackEngine>,
    command_tx: Sender<EngineCommand>,
    running: Arc<AtomicBool>,
    snapshot_cache: Arc<Mutex<EngineSnapshot>>,
    events: Receiver<PlaybackEvent>,
}

impl PlaybackController {
    /// Spawn the engine worker thread and return the controller handle.
    pub fn spawn(engine: Arc<dyn PlaybackEngine>) -> Self {
        let (command_tx, command_rx) = unbounded::<EngineCommand>();
        let running = Arc::new(AtomicBool::new(true));
        let snapshot_cache = Arc::new(Mutex::new(EngineSnapshot::default()));

        let engine_for_thread = engine.clone();
        let running_for_thread = running.clone();
        let cache_for_thread = snapshot_cache.clone();
        std::thread::Builder::new()
            .name("noir-engine".into())
            .spawn(move || loop {
                if !running_for_thread.load(Ordering::Relaxed) {
                    engine_for_thread.request_shutdown();
                    break;
                }
                if let Ok(cmd) = command_rx.recv_timeout(std::time::Duration::from_millis(50)) {
                    match cmd {
                        EngineCommand::Shutdown => {
                            running_for_thread.store(false, Ordering::Relaxed);
                            engine_for_thread.request_shutdown();
                            break;
                        }
                        other => engine_for_thread.send(other),
                    }
                }
                *cache_for_thread.lock().unwrap() = engine_for_thread.snapshot();
            })
            .expect("failed to spawn engine thread");

        Self {
            engine: engine.clone(),
            command_tx,
            running,
            snapshot_cache,
            events: engine.subscribe(),
        }
    }

    /// The engine behind this controller (for capability checks, rendering).
    pub fn engine(&self) -> &Arc<dyn PlaybackEngine> {
        &self.engine
    }

    /// Queue a command without blocking.
    pub fn execute(&self, command: EngineCommand) {
        if let Err(e) = self.command_tx.send(command) {
            warn!("engine command dropped: {e}");
        }
    }

    pub fn open(&self, path: impl Into<String>, start_position: Option<f64>) {
        self.execute(EngineCommand::Open {
            path: path.into(),
            start_position,
        });
    }

    pub fn toggle_play(&self) {
        self.execute(EngineCommand::TogglePlay);
    }

    pub fn stop(&self) {
        self.execute(EngineCommand::Stop);
    }

    pub fn seek_relative(&self, offset: f64) {
        self.execute(EngineCommand::SeekRelative { offset });
    }

    pub fn seek_absolute(&self, position: f64) {
        self.execute(EngineCommand::SeekAbsolute { position });
    }

    pub fn set_speed(&self, speed: f64) {
        self.execute(EngineCommand::SetSpeed { speed });
    }

    pub fn set_volume(&self, volume: u32) {
        self.execute(EngineCommand::SetVolume { volume });
    }

    pub fn toggle_mute(&self) {
        let muted = !self.snapshot().muted;
        self.execute(EngineCommand::SetMuted { muted });
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.command_tx.send(EngineCommand::Shutdown);
    }

    /// Latest engine snapshot, refreshed by the worker thread. Cheap.
    pub fn snapshot(&self) -> EngineSnapshot {
        self.snapshot_cache.lock().unwrap().clone()
    }

    /// Current playback state, extracted from the snapshot.
    pub fn state(&self) -> PlaybackState {
        self.snapshot().state
    }

    /// Drain pending engine events (EndOfFile, errors, ...) for UI reaction.
    pub fn drain_events(&self) -> Vec<PlaybackEvent> {
        self.events.try_iter().collect()
    }
}

impl Drop for PlaybackController {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockEngine;

    #[test]
    fn controller_spawns_and_shuts_down() {
        let engine = Arc::new(MockEngine::new());
        let controller = PlaybackController::spawn(engine);
        assert_eq!(controller.state(), PlaybackState::Idle);
        controller.open("/tmp/test.mp4", None);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let snap = controller.snapshot();
        assert_eq!(snap.state, PlaybackState::Playing);
        assert_eq!(snap.current_path.as_deref(), Some("/tmp/test.mp4"));
        controller.shutdown();
    }

    #[test]
    fn events_are_drainable() {
        let controller = PlaybackController::spawn(Arc::new(MockEngine::new()));
        controller.open("/tmp/x.mkv", None);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let events = controller.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, PlaybackEvent::FileLoaded { .. })));
        controller.shutdown();
    }
}

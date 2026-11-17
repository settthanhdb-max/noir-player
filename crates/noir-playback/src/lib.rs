//! Playback engine abstraction.
//!
//! The UI talks to engines exclusively through [`PlaybackEngine`] and the
//! [`crate::controller::PlaybackController`] command channel, never through
//! backend-specific APIs.

pub mod capabilities;
pub mod controller;
pub mod engine;
pub mod mock;
#[cfg(feature = "mpv")]
pub mod mpv;
pub mod settings_map;

pub use capabilities::Capabilities;
pub use controller::PlaybackController;
pub use engine::{EngineSnapshot, PlaybackEngine};

/// Create the best available engine for this build.
///
/// With the `mpv` feature enabled and a working native libmpv, this returns
/// the real mpv backend; otherwise it falls back to the mock engine, which
/// reports its limitation honestly instead of pretending to play media.
pub fn create_default_engine() -> std::sync::Arc<dyn PlaybackEngine> {
    #[cfg(feature = "mpv")]
    {
        if let Some(engine) = mpv::MpvEngine::try_new() {
            return std::sync::Arc::new(engine);
        }
    }
    std::sync::Arc::new(mock::MockEngine::new())
}

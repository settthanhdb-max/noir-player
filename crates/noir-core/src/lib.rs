//! NOIR Player core types.
//!
//! This crate is the shared vocabulary of the application: data models,
//! playback state, events and errors. It has no dependency on any GUI
//! framework or media backend, so every other crate can depend on it.

pub mod errors;
pub mod events;
pub mod models;
pub mod state;

pub use errors::{NoirError, Result};
pub use events::{EngineEvent, PlaybackEvent, UiEvent};
pub use models::{ChapterInfo, MediaItem, MediaKind, MediaStats, TrackInfo, TrackKind};
pub use state::PlaybackState;

//! Explicit playback state machine.
//!
//! ```text
//! Idle -> Loading -> Playing <-> Paused -> Ended
//!                       |          |
//!                  Buffering <----+
//! any -> Error
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackState {
    #[default]
    Idle,
    Loading,
    Playing,
    Paused,
    Buffering,
    Ended,
    Error(String),
}

impl PlaybackState {
    /// Whether media is actively decoded and rendered (or would be if paused).
    pub fn has_media(&self) -> bool {
        matches!(
            self,
            PlaybackState::Loading
                | PlaybackState::Playing
                | PlaybackState::Paused
                | PlaybackState::Buffering
                | PlaybackState::Ended
                | PlaybackState::Error(_)
        )
    }

    pub fn is_playing(&self) -> bool {
        matches!(self, PlaybackState::Playing | PlaybackState::Buffering)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, PlaybackState::Paused)
    }

    pub fn is_busy(&self) -> bool {
        matches!(self, PlaybackState::Loading | PlaybackState::Buffering)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, PlaybackState::Ended | PlaybackState::Error(_))
    }

    /// Validate a state transition. Illegal transitions are reported as `None`
    /// so callers can log and ignore engine quirks instead of corrupting UI state.
    pub fn can_transition_to(&self, next: &PlaybackState) -> bool {
        use PlaybackState::*;
        if next == self {
            return true;
        }
        matches!(
            (self, next),
            // Anything can go to Error.
            (_, Error(_))
                // Anything with media can be stopped back to Idle.
                | (Loading | Playing | Paused | Buffering | Ended | Error(_), Idle)
                // Loading phase.
                | (Idle, Loading)
                | (Loading, Playing)
                | (Loading, Paused)
                | (Loading, Buffering)
                | (Loading, Ended)
                // Normal playback.
                | (Playing, Paused)
                | (Playing, Buffering)
                | (Playing, Ended)
                | (Playing, Loading)
                // Paused.
                | (Paused, Playing)
                | (Paused, Buffering)
                | (Paused, Ended)
                | (Paused, Loading)
                // Buffering recovery.
                | (Buffering, Playing)
                | (Buffering, Paused)
                | (Buffering, Ended)
                // Ended -> replay / next file.
                | (Ended, Loading)
                | (Ended, Playing)
                | (Ended, Paused)
        )
    }

    pub fn label(&self) -> String {
        match self {
            PlaybackState::Idle => "Idle".into(),
            PlaybackState::Loading => "Loading".into(),
            PlaybackState::Playing => "Playing".into(),
            PlaybackState::Paused => "Paused".into(),
            PlaybackState::Buffering => "Buffering".into(),
            PlaybackState::Ended => "Ended".into(),
            PlaybackState::Error(e) => format!("Error: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_transitions() {
        assert!(PlaybackState::Idle.can_transition_to(&PlaybackState::Loading));
        assert!(PlaybackState::Loading.can_transition_to(&PlaybackState::Playing));
        assert!(PlaybackState::Playing.can_transition_to(&PlaybackState::Paused));
        assert!(PlaybackState::Paused.can_transition_to(&PlaybackState::Playing));
        assert!(PlaybackState::Playing.can_transition_to(&PlaybackState::Ended));
        assert!(PlaybackState::Ended.can_transition_to(&PlaybackState::Loading));
        assert!(PlaybackState::Playing.can_transition_to(&PlaybackState::Error("x".into())));
    }

    #[test]
    fn illegal_transitions() {
        assert!(!PlaybackState::Idle.can_transition_to(&PlaybackState::Playing));
        assert!(!PlaybackState::Ended.can_transition_to(&PlaybackState::Buffering));
        assert!(!PlaybackState::Error("x".into()).can_transition_to(&PlaybackState::Playing));
    }

    #[test]
    fn predicates() {
        assert!(PlaybackState::Playing.is_playing());
        assert!(PlaybackState::Buffering.is_playing());
        assert!(!PlaybackState::Paused.is_playing());
        assert!(PlaybackState::Paused.is_paused());
        assert!(PlaybackState::Idle.can_transition_to(&PlaybackState::Idle));
        assert!(!PlaybackState::Idle.has_media());
        assert!(PlaybackState::Error("x".into()).has_media());
    }
}

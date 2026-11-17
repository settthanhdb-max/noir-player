//! Playback history policy: decides when sessions are recorded and computes
//! completion. The database layer is storage-only; this module owns policy.

use noir_core::events::HistoryEntry;
use std::time::SystemTime;

/// Policy inputs for recording one session.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HistoryPolicy {
    pub enabled: bool,
    pub remember_position: bool,
    /// Do not record sessions shorter than this (seconds).
    pub min_duration_s: f64,
}

impl Default for HistoryPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            remember_position: true,
            min_duration_s: 5.0,
        }
    }
}

/// Decide whether a stopped session should be recorded, and build the entry.
/// Returns `None` when policy says "don't record".
pub fn build_entry(
    policy: HistoryPolicy,
    path: &str,
    title: &str,
    position: f64,
    duration: Option<f64>,
) -> Option<HistoryEntry> {
    if !policy.enabled {
        return None;
    }
    if position < policy.min_duration_s {
        return None;
    }
    let completion = match duration {
        Some(d) if d > 0.0 => (position / d).clamp(0.0, 1.0) as f32,
        _ => 0.0,
    };
    Some(HistoryEntry {
        path: path.to_string(),
        title: title.to_string(),
        last_played: SystemTime::now(),
        position: if policy.remember_position {
            position
        } else {
            0.0
        },
        duration,
        completion,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_policy() {
        let policy = HistoryPolicy::default();
        assert!(build_entry(policy, "/a", "a", 30.0, Some(100.0)).is_some());
        assert!(build_entry(policy, "/a", "a", 2.0, Some(100.0)).is_none());
        let disabled = HistoryPolicy {
            enabled: false,
            ..Default::default()
        };
        assert!(build_entry(disabled, "/a", "a", 30.0, Some(100.0)).is_none());
    }

    #[test]
    fn completion_ratio() {
        let e = build_entry(HistoryPolicy::default(), "/a", "a", 50.0, Some(200.0)).unwrap();
        assert!((e.completion - 0.25).abs() < 1e-6);
        let e = build_entry(HistoryPolicy::default(), "/a", "a", 500.0, Some(200.0)).unwrap();
        assert!((e.completion - 1.0).abs() < 1e-6);
    }

    #[test]
    fn position_not_remembered_when_disabled() {
        let policy = HistoryPolicy {
            remember_position: false,
            ..Default::default()
        };
        let e = build_entry(policy, "/a", "a", 30.0, Some(100.0)).unwrap();
        assert_eq!(e.position, 0.0);
    }
}

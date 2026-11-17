//! Named settings profiles: partial overrides stored as JSON patches.

use crate::schema::Settings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A named profile holding dotted-key overrides, e.g.
/// `{"playback.default_speed": 1.5}`. Profiles never need to be exhaustive,
/// so they survive schema changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SettingsProfile {
    pub name: String,
    pub overrides: BTreeMap<String, serde_json::Value>,
}

impl SettingsProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            overrides: Default::default(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.overrides.insert(key.into(), value);
    }

    /// Apply this profile's overrides onto `settings`. Unknown keys are
    /// reported, never silently ignored.
    pub fn apply(&self, settings: &mut Settings) -> Result<(), String> {
        for (key, value) in &self.overrides {
            let (category, field) = key
                .split_once('.')
                .ok_or_else(|| format!("invalid key '{key}'"))?;
            let category_json = category_to_json(settings, category)?;
            let mut obj = match category_json {
                serde_json::Value::Object(o) => o,
                _ => return Err(format!("category '{category}' is not a table")),
            };
            match obj.get_mut(field) {
                Some(slot) => coerce_into(slot, value.clone())?,
                None => return Err(format!("unknown field '{field}' in category '{category}'")),
            }
            let merged = serde_json::from_value(serde_json::Value::Object(obj))
                .map_err(|e| format!("failed to apply '{key}': {e}"))?;
            assign_category(settings, category, merged)?;
        }
        Ok(())
    }
}

fn coerce_into(slot: &mut serde_json::Value, value: serde_json::Value) -> Result<(), String> {
    let coerced = match (&*slot, value) {
        (serde_json::Value::Number(_), serde_json::Value::Number(n)) => {
            serde_json::Value::Number(n)
        }
        (serde_json::Value::Number(_), serde_json::Value::String(s)) => {
            let n: serde_json::Number = s.parse().map_err(|_| format!("'{s}' is not a number"))?;
            serde_json::Value::Number(n)
        }
        (serde_json::Value::Bool(_), serde_json::Value::Bool(b)) => serde_json::Value::Bool(b),
        (serde_json::Value::Bool(_), serde_json::Value::String(s)) => {
            serde_json::Value::Bool(matches!(s.as_str(), "true" | "yes" | "1"))
        }
        (serde_json::Value::String(_), serde_json::Value::String(s)) => {
            serde_json::Value::String(s)
        }
        (serde_json::Value::Array(_), serde_json::Value::Array(a)) => serde_json::Value::Array(a),
        (_, value) => value,
    };
    *slot = coerced;
    Ok(())
}

fn category_to_json(s: &mut Settings, name: &str) -> Result<serde_json::Value, String> {
    match name {
        "general" => serde_json::to_value(&s.general).map_err(|e| e.to_string()),
        "appearance" => serde_json::to_value(&s.appearance).map_err(|e| e.to_string()),
        "playback" => serde_json::to_value(&s.playback).map_err(|e| e.to_string()),
        "video" => serde_json::to_value(&s.video).map_err(|e| e.to_string()),
        "audio" => serde_json::to_value(&s.audio).map_err(|e| e.to_string()),
        "subtitles" => serde_json::to_value(&s.subtitles).map_err(|e| e.to_string()),
        "network" => serde_json::to_value(&s.network).map_err(|e| e.to_string()),
        "playlist" => serde_json::to_value(&s.playlist).map_err(|e| e.to_string()),
        "input" => serde_json::to_value(&s.input).map_err(|e| e.to_string()),
        "advanced" => serde_json::to_value(&s.advanced).map_err(|e| e.to_string()),
        _ => Err(format!("unknown category '{name}'")),
    }
}

fn assign_category(s: &mut Settings, name: &str, value: serde_json::Value) -> Result<(), String> {
    macro_rules! assign {
        ($field:ident) => {{
            s.$field = serde_json::from_value(value).map_err(|e| e.to_string())?;
        }};
    }
    match name {
        "general" => {
            assign!(general);
            Ok(())
        }
        "appearance" => {
            assign!(appearance);
            Ok(())
        }
        "playback" => {
            assign!(playback);
            Ok(())
        }
        "video" => {
            assign!(video);
            Ok(())
        }
        "audio" => {
            assign!(audio);
            Ok(())
        }
        "subtitles" => {
            assign!(subtitles);
            Ok(())
        }
        "network" => {
            assign!(network);
            Ok(())
        }
        "playlist" => {
            assign!(playlist);
            Ok(())
        }
        "input" => {
            assign!(input);
            Ok(())
        }
        "advanced" => {
            assign!(advanced);
            Ok(())
        }
        _ => Err(format!("unknown category '{name}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_applies_overrides() {
        let mut s = Settings::defaults();
        let mut p = SettingsProfile::new("Cinema");
        p.set("playback.default_speed", serde_json::json!(1.25));
        p.set("playback.default_volume", serde_json::json!("85"));
        p.set("appearance.compact_mode", serde_json::json!(true));
        p.apply(&mut s).unwrap();
        assert!((s.playback.default_speed - 1.25).abs() < f32::EPSILON);
        assert_eq!(s.playback.default_volume, 85);
        assert!(s.appearance.compact_mode);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let mut s = Settings::defaults();
        let mut p = SettingsProfile::new("Bad");
        p.set("playback.nonexistent", serde_json::json!(1));
        assert!(p.apply(&mut s).is_err());
        let mut p = SettingsProfile::new("Bad2");
        p.set("notacategory.x", serde_json::json!(1));
        assert!(p.apply(&mut s).is_err());
    }

    #[test]
    fn profile_serializes() {
        let mut p = SettingsProfile::new("Night");
        p.set("appearance.theme", serde_json::json!(0));
        let json = serde_json::to_string(&p).unwrap();
        let back: SettingsProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }
}

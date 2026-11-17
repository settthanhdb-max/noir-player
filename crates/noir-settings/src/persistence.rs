//! Settings persistence: versioned TOML file with atomic writes.

use crate::migration;
use crate::schema::Settings;
use crate::{SettingsError, SETTINGS_VERSION};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Owns the settings file location and load/save operations.
#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load settings, applying migrations for older versions. Any failure
    /// falls back to defaults rather than crashing the application.
    pub fn load(&self) -> Settings {
        match self.try_load() {
            Ok(s) => s,
            Err(e) => {
                warn!("settings load failed ({}), using defaults", e);
                Settings::defaults()
            }
        }
    }

    pub fn try_load(&self) -> Result<Settings, SettingsError> {
        if !self.path.exists() {
            return Ok(Settings::defaults());
        }
        let text = fs::read_to_string(&self.path)?;
        let mut settings: Settings = toml::from_str(&text)?;
        migration::migrate(&mut settings);
        settings.version = SETTINGS_VERSION;
        Ok(settings)
    }

    /// Save atomically: write to a temp file, then rename over the target.
    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(settings)?;
        let tmp = self.path.with_extension("toml.tmp");
        fs::write(&tmp, text.as_bytes())?;
        match fs::rename(&tmp, &self.path) {
            Ok(()) => {
                info!("settings saved to {}", self.path.display());
                Ok(())
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                Err(e.into())
            }
        }
    }

    /// Export settings to an arbitrary path (backup / import sharing).
    pub fn export_to(&self, settings: &Settings, target: &Path) -> Result<(), SettingsError> {
        let text = toml::to_string_pretty(settings)?;
        fs::write(target, text.as_bytes())?;
        Ok(())
    }

    /// Import settings from a file, validating before returning them.
    pub fn import_from(&self, source: &Path) -> Result<Settings, SettingsError> {
        let text = fs::read_to_string(source)?;
        let mut settings: Settings = toml::from_str(&text)?;
        migration::migrate(&mut settings);
        settings.version = SETTINGS_VERSION;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("noir-settings-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn save_load_roundtrip() {
        let path = temp_dir().join("settings.toml");
        let store = SettingsStore::new(&path);
        let mut s = Settings::defaults();
        s.playback.default_speed = 1.5;
        s.general.language = 1;
        store.save(&s).unwrap();
        let loaded = store.try_load().unwrap();
        assert!((loaded.playback.default_speed - 1.5).abs() < f32::EPSILON);
        assert_eq!(loaded.general.language, 1);
        assert_eq!(loaded.version, SETTINGS_VERSION);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_returns_defaults() {
        let store = SettingsStore::new(temp_dir().join("nonexistent.toml"));
        let s = store.try_load().unwrap();
        assert_eq!(s, Settings::defaults());
    }

    #[test]
    fn corrupt_file_falls_back() {
        let path = temp_dir().join("corrupt.toml");
        fs::write(&path, "this is [not toml").unwrap();
        let store = SettingsStore::new(&path);
        assert!(store.try_load().is_err());
        assert_eq!(store.load(), Settings::defaults());
        let _ = fs::remove_file(&path);
    }
}

//! Standard platform directories (config, data, cache, pictures).

use directories::{BaseDirs, ProjectDirs, UserDirs};
use std::path::PathBuf;

/// Resolved locations used across the application.
#[derive(Debug, Clone)]
pub struct PlatformPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub pictures_dir: Option<PathBuf>,
    pub home_dir: Option<PathBuf>,
}

impl PlatformPaths {
    /// Resolve directories for the `noir-player` application identity.
    pub fn resolve() -> Self {
        let base = BaseDirs::new();
        let proj = ProjectDirs::from("app", "noir", "noir-player");
        Self {
            config_dir: proj
                .as_ref()
                .map(|p| p.config_dir().to_path_buf())
                .or_else(|| base.as_ref().map(|b| b.home_dir().join(".noir-player")))
                .unwrap_or_else(|| PathBuf::from(".noir-player")),
            data_dir: proj
                .as_ref()
                .map(|p| p.data_dir().to_path_buf())
                .or_else(|| base.as_ref().map(|b| b.home_dir().join(".noir-player")))
                .unwrap_or_else(|| PathBuf::from(".noir-player")),
            cache_dir: proj
                .as_ref()
                .map(|p| p.cache_dir().to_path_buf())
                .or_else(|| {
                    base.as_ref()
                        .map(|b| b.home_dir().join(".noir-player/cache"))
                })
                .unwrap_or_else(|| PathBuf::from(".noir-player/cache")),
            pictures_dir: UserDirs::new().and_then(|u| u.picture_dir().map(|p| p.to_path_buf())),
            home_dir: base.as_ref().map(|b| b.home_dir().to_path_buf()),
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.toml")
    }

    pub fn database_file(&self) -> PathBuf {
        self.data_dir.join("library.db")
    }

    pub fn log_file(&self) -> PathBuf {
        self.data_dir.join("noir-player.log")
    }

    /// Ensure config/data/cache directories exist. Returns I/O errors
    /// instead of silently continuing without persistence.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_resolve() {
        let p = PlatformPaths::resolve();
        assert!(p.settings_file().ends_with("settings.toml"));
        assert!(p.database_file().ends_with("library.db"));
    }
}

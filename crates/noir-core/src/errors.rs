//! Application-wide error type.

use thiserror::Error;

/// Result alias used across NOIR Player crates.
pub type Result<T> = std::result::Result<T, NoirError>;

#[derive(Debug, Error)]
pub enum NoirError {
    #[error("media file not found: {path}")]
    MediaNotFound { path: String },

    #[error("unsupported media: {reason}")]
    UnsupportedMedia { reason: String },

    #[error("playback engine error: {0}")]
    Engine(String),

    #[error("renderer error: {0}")]
    Renderer(String),

    #[error("audio error: {0}")]
    Audio(String),

    #[error("subtitle error: {0}")]
    Subtitle(String),

    #[error("settings error: {0}")]
    Settings(String),

    #[error("invalid setting value for '{key}': {reason}")]
    InvalidSetting { key: String, reason: String },

    #[error("library/database error: {0}")]
    Database(String),

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("configuration parse error: {0}")]
    Parse(String),

    #[error("operation cancelled")]
    Cancelled,
}

impl NoirError {
    pub fn engine(msg: impl Into<String>) -> Self {
        Self::Engine(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    pub fn settings(msg: impl Into<String>) -> Self {
        Self::Settings(msg.into())
    }

    /// True when the error is transient and a retry may succeed
    /// (network interruptions, temporary device loss).
    pub fn is_retryable(&self) -> bool {
        matches!(self, NoirError::Engine(m) if m.contains("network") || m.contains("timeout"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_network_errors() {
        assert!(NoirError::engine("network timeout").is_retryable());
        assert!(!NoirError::engine("cannot open file").is_retryable());
    }

    #[test]
    fn display_is_human_readable() {
        let e = NoirError::MediaNotFound {
            path: "C:/missing.mkv".into(),
        };
        assert!(e.to_string().contains("C:/missing.mkv"));
    }
}

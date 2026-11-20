//! NOIR Player settings system.
//!
//! Strongly typed, versioned, persistable settings with a descriptor layer
//! that lets the UI render every setting generically (search, tooltips,
//! reset-to-default, advanced flags) without duplicating widget code.

pub mod migration;
pub mod persistence;
pub mod profiles;
pub mod schema;
pub mod validation;
pub mod value;

pub use persistence::SettingsStore;
pub use schema::input::{
    default_bindings, ActionBinding, InputSettings, KeyBinding, KeyCombo, Modifiers,
    ShortcutRegistry,
};
pub use schema::Settings;
pub use value::{SettingEntry, SettingKind, SettingValue};

/// Current settings schema version. Bump when making a breaking change and
/// add a migration step in [`migration::migrate`].
pub const SETTINGS_VERSION: u32 = 3;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("unknown setting key: {0}")]
    UnknownKey(String),
    #[error("validation: {0}")]
    Validation(String),
}

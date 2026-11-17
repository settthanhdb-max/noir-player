//! Category 10 — Advanced.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const LOG_LEVELS: [&str; 5] = ["Error", "Warn", "Info", "Debug", "Trace"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedSettings {
    pub advanced_mode: bool,
    /// Raw backend options, one `key=value` per line. Validated before apply.
    pub custom_mpv_options: String,
    /// Explicit decoder override, empty = automatic.
    pub decoder: String,
    pub cache_enabled: bool,
    pub render_diagnostics: bool,
    /// Index into [`LOG_LEVELS`].
    pub log_level: usize,
    pub log_to_file: bool,
    pub show_capabilities: bool,
    pub performance_monitor: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            advanced_mode: false,
            custom_mpv_options: String::new(),
            decoder: String::new(),
            cache_enabled: true,
            render_diagnostics: false,
            log_level: 2,
            log_to_file: false,
            show_capabilities: false,
            performance_monitor: false,
        }
    }
}

impl AdvancedSettings {
    /// Parse `custom_mpv_options` into key/value pairs.
    ///
    /// Accepts `key=value` lines and `--key value` lines. Rejects empty keys,
    /// non-ASCII option names and unbalanced quotes, so a bad user value can
    /// never crash the backend.
    pub fn parse_custom_options(&self) -> Result<Vec<(String, String)>, String> {
        let mut out = Vec::new();
        for (line_no, raw) in self.custom_mpv_options.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let line = line.trim_start_matches("--");
            let (key, value) = match line.split_once('=') {
                Some((k, v)) => (k.trim(), v.trim()),
                None => match line.split_once(' ') {
                    Some((k, v)) => (k.trim(), v.trim()),
                    None => (line, ""),
                },
            };
            if key.is_empty()
                || !key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'))
            {
                return Err(format!(
                    "line {}: invalid option name '{}'",
                    line_no + 1,
                    key
                ));
            }
            if value.matches('"').count() % 2 != 0 {
                return Err(format!("line {}: unbalanced quotes", line_no + 1));
            }
            out.push((key.to_string(), value.trim_matches('"').to_string()));
        }
        Ok(out)
    }

    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "advanced.advanced_mode",
                "Advanced configuration mode",
                "Show advanced and backend-specific options.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.advanced_mode),
            ),
            SettingEntry::new(
                "advanced.custom_mpv_options",
                "Custom mpv options",
                "One key=value per line, applied to the engine after validation.",
                SettingKind::Text,
                SettingValue::Text(&mut self.custom_mpv_options),
            )
            .advanced(),
            SettingEntry::new(
                "advanced.decoder",
                "Decoder selection",
                "Force a specific decoder, empty = automatic.",
                SettingKind::Text,
                SettingValue::Text(&mut self.decoder),
            )
            .advanced(),
            SettingEntry::new(
                "advanced.cache_enabled",
                "Cache configuration",
                "Enable the demuxer cache.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.cache_enabled),
            )
            .advanced(),
            SettingEntry::new(
                "advanced.render_diagnostics",
                "Rendering diagnostics",
                "Log renderer timing and frame statistics.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.render_diagnostics),
            )
            .advanced(),
            SettingEntry::new(
                "advanced.log_level",
                "Logging verbosity",
                "Minimum log level written to the log output.",
                SettingKind::Dropdown {
                    options: LOG_LEVELS.to_vec(),
                },
                SettingValue::Choice(&mut self.log_level),
            ),
            SettingEntry::new(
                "advanced.log_to_file",
                "Log to file",
                "Write logs next to the configuration directory.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.log_to_file),
            ),
            SettingEntry::new(
                "advanced.show_capabilities",
                "Hardware capability information",
                "Show detected decoder and output capabilities.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_capabilities),
            ),
            SettingEntry::new(
                "advanced.performance_monitor",
                "Performance monitoring",
                "Show live CPU/frame performance metrics.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.performance_monitor),
            )
            .advanced(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_value_lines() {
        let mut a = AdvancedSettings::default();
        a.custom_mpv_options = "--volume-max=130\nsub-auto=fuzzy\n# comment\n".into();
        let opts = a.parse_custom_options().unwrap();
        assert_eq!(
            opts,
            vec![
                ("volume-max".into(), "130".into()),
                ("sub-auto".into(), "fuzzy".into())
            ]
        );
    }

    #[test]
    fn rejects_invalid_option_names() {
        let mut a = AdvancedSettings::default();
        a.custom_mpv_options = "bad key!=1".into();
        assert!(a.parse_custom_options().is_err());
        a.custom_mpv_options = "key=\"unbalanced".into();
        assert!(a.parse_custom_options().is_err());
    }
}

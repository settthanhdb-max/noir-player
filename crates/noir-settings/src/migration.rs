//! Settings schema migrations.
//!
//! Each migration upgrades an older persisted document in place. Migrations
//! run in order until the document reaches [`SETTINGS_VERSION`].

use crate::schema::Settings;
use tracing::debug;

/// Apply all pending migrations to `settings`.
pub fn migrate(settings: &mut Settings) {
    if settings.version < 2 {
        migrate_v1_to_v2(settings);
    }
    if settings.version < 3 {
        migrate_v2_to_v3(settings);
    }
}

/// v1 -> v2: the old `general.lang` string field became `general.language`
/// (an index into the language list) and the old `appearance.ui_scale_pct`
/// integer became the `ui_scale` float.
fn migrate_v1_to_v2(settings: &mut Settings) {
    debug!("migrating settings v1 -> v2");
    // v1 stored the UI scale as a percentage (125.0 == 1.25x).
    if settings.appearance.ui_scale > 2.0 {
        settings.appearance.ui_scale = (settings.appearance.ui_scale / 100.0).clamp(0.75, 2.0);
    }
    settings.general.language = settings.general.language.min(1);
}

/// v2 -> v3: the control bar landed and the auto-hide delay default became
/// 4 s (was 10 s). Existing documents that still carry the old default are
/// moved to the new one; values the user actually changed (anything but
/// 10.0) are preserved.
fn migrate_v2_to_v3(settings: &mut Settings) {
    debug!("migrating settings v2 -> v3");
    if settings.appearance.autohide_delay == 10.0 {
        settings.appearance.autohide_delay = 4.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::schema::Settings;

    #[test]
    fn v1_document_is_upgraded() {
        let text = "version = 1\n[appearance]\nui_scale = 125.0\n";
        let mut s: Settings = toml::from_str(text).unwrap();
        crate::migration::migrate(&mut s);
        assert!((s.appearance.ui_scale - 1.25).abs() < 0.01);
    }
}

//! Cross-field settings validation and normalization.

use crate::schema::Settings;
use noir_core::errors::NoirError;

/// Validate cross-field invariants. Returns every violation found.
pub fn validate(settings: &Settings) -> Vec<NoirError> {
    let mut errors = Vec::new();
    let p = &settings.playback;

    if p.min_speed <= 0.0 || p.min_speed >= p.max_speed {
        errors.push(NoirError::InvalidSetting {
            key: "playback.min_speed".into(),
            reason: "minimum speed must be positive and below maximum speed".into(),
        });
    }
    if p.max_speed > 8.0 {
        errors.push(NoirError::InvalidSetting {
            key: "playback.max_speed".into(),
            reason: "maximum speed cannot exceed 8x".into(),
        });
    }
    if p.speed_increment <= 0.0 {
        errors.push(NoirError::InvalidSetting {
            key: "playback.speed_increment".into(),
            reason: "speed increment must be positive".into(),
        });
    }
    if p.resume_threshold_s < 0.0 {
        errors.push(NoirError::InvalidSetting {
            key: "playback.resume_threshold_s".into(),
            reason: "resume threshold cannot be negative".into(),
        });
    }
    if p.seek_forward_s <= 0.0 || p.seek_backward_s <= 0.0 {
        errors.push(NoirError::InvalidSetting {
            key: "playback.seek_forward_s".into(),
            reason: "seek intervals must be positive".into(),
        });
    }

    let a = &settings.appearance;
    if !(0.5..=1.0).contains(&a.window_opacity) {
        errors.push(NoirError::InvalidSetting {
            key: "appearance.window_opacity".into(),
            reason: "opacity must be between 0.5 and 1.0".into(),
        });
    }
    if a.autohide_delay < 0.5 {
        errors.push(NoirError::InvalidSetting {
            key: "appearance.autohide_delay".into(),
            reason: "auto-hide delay must be at least 0.5s".into(),
        });
    }

    let v = &settings.video;
    if !(0.5..=4.0).contains(&v.zoom) {
        errors.push(NoirError::InvalidSetting {
            key: "video.zoom".into(),
            reason: "zoom must be between 0.5 and 4.0".into(),
        });
    }
    if v.screenshot_quality > 100 {
        errors.push(NoirError::InvalidSetting {
            key: "video.screenshot_quality".into(),
            reason: "quality cannot exceed 100".into(),
        });
    }
    let s = &settings.subtitles;
    for (key, color) in [
        ("subtitles.font_color", &s.font_color),
        ("subtitles.outline_color", &s.outline_color),
    ] {
        if let Err(reason) = validate_color(color) {
            errors.push(NoirError::InvalidSetting {
                key: key.into(),
                reason,
            });
        }
    }
    if !(0.0..=100.0).contains(&s.vertical_position) {
        errors.push(NoirError::InvalidSetting {
            key: "subtitles.vertical_position".into(),
            reason: "vertical position must be 0-100".into(),
        });
    }

    let ad = &settings.audio;
    for (i, gain) in ad.eq_gains.iter().enumerate() {
        if !(-12.0..=12.0).contains(gain) {
            errors.push(NoirError::InvalidSetting {
                key: format!("audio.eq_band_{}", i),
                reason: "equalizer gain must be between -12 and +12 dB".into(),
            });
        }
    }

    let n = &settings.network;
    if n.timeout_s == 0 {
        errors.push(NoirError::InvalidSetting {
            key: "network.timeout_s".into(),
            reason: "timeout must be greater than zero".into(),
        });
    }
    if n.cache_size_mib > 4096 {
        errors.push(NoirError::InvalidSetting {
            key: "network.cache_size_mib".into(),
            reason: "cache size cannot exceed 4096 MiB".into(),
        });
    }

    let adv = &settings.advanced;
    if let Err(msg) = adv.parse_custom_options() {
        errors.push(NoirError::InvalidSetting {
            key: "advanced.custom_mpv_options".into(),
            reason: msg,
        });
    }

    errors
}

/// Validate a `#RRGGBB` / `#AARRGGBB` color string (or empty = default).
pub fn validate_color(color: &str) -> Result<(), String> {
    if color.is_empty() {
        return Ok(());
    }
    let body = color.strip_prefix('#').ok_or("color must start with '#'")?;
    if body.len() != 6 && body.len() != 8 {
        return Err("color must be #RRGGBB or #AARRGGBB".into());
    }
    if !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("color contains non-hexadecimal characters".into());
    }
    Ok(())
}

/// Clamp values back into legal ranges without reporting errors; used when
/// importing or loading documents that may be slightly out of range.
pub fn normalize(settings: &mut Settings) {
    settings.playback.max_speed = settings.playback.max_speed.clamp(1.0, 8.0);
    settings.playback.min_speed = settings
        .playback
        .min_speed
        .clamp(0.1, settings.playback.max_speed);
    settings.playback.speed_increment = settings.playback.speed_increment.clamp(0.05, 0.5);
    settings.playback.default_speed = settings
        .playback
        .clamp_speed(settings.playback.default_speed);
    settings.playback.default_volume = settings
        .playback
        .clamp_volume(settings.playback.default_volume);
    settings.appearance.ui_scale = settings.appearance.ui_scale.clamp(0.75, 2.0);
    settings.appearance.window_opacity = settings.appearance.window_opacity.clamp(0.5, 1.0);
    settings.appearance.autohide_delay = settings.appearance.autohide_delay.clamp(0.5, 10.0);
    settings.video.zoom = settings.video.zoom.clamp(0.5, 4.0);
    settings.video.screenshot_quality = settings.video.screenshot_quality.clamp(50, 100);
    settings.subtitles.vertical_position = settings.subtitles.vertical_position.clamp(0.0, 100.0);
    settings
        .audio
        .eq_gains
        .iter_mut()
        .for_each(|g| *g = g.clamp(-12.0, 12.0));
    settings.network.timeout_s = settings.network.timeout_s.max(1);
    settings.network.cache_size_mib = settings.network.cache_size_mib.clamp(4, 4096);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Settings;

    #[test]
    fn defaults_are_valid() {
        assert!(validate(&Settings::defaults()).is_empty());
    }

    #[test]
    fn detects_speed_inversion() {
        let mut s = Settings::defaults();
        s.playback.min_speed = 5.0;
        s.playback.max_speed = 2.0;
        let errs = validate(&s);
        assert!(errs.iter().any(|e| e.to_string().contains("minimum speed")));
    }

    #[test]
    fn detects_bad_colors() {
        assert!(validate_color("#FF00FF").is_ok());
        assert!(validate_color("#80FF00FF").is_ok());
        assert!(validate_color("FF00FF").is_err());
        assert!(validate_color("#GGGGGG").is_err());
        assert!(validate_color("").is_ok());
    }

    #[test]
    fn normalize_repairs_out_of_range() {
        let mut s = Settings::defaults();
        s.playback.max_speed = 99.0;
        s.playback.default_volume = 500;
        s.video.zoom = 42.0;
        normalize(&mut s);
        assert!(validate(&s).is_empty());
    }

    #[test]
    fn custom_options_errors_are_reported() {
        let mut s = Settings::defaults();
        s.advanced.custom_mpv_options = "=1".into();
        let errs = validate(&s);
        assert!(errs
            .iter()
            .any(|e| e.to_string().contains("custom_mpv_options")));
    }
}

//! NOIR Player monochrome theme.
//!
//! The palettes are the product identity: near-black surfaces, white
//! highlights, no color accents. Both themes are always available.

use egui::{Color32, Context, CornerRadius, FontId, Margin, Stroke, TextStyle, Vec2, Visuals};
use noir_settings::schema::AppearanceSettings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Dark,
    Light,
}

impl Palette {
    pub fn from_theme_index(index: usize, system_dark: bool) -> Self {
        match index {
            1 => Self::Light,
            2 => {
                if system_dark {
                    Self::Dark
                } else {
                    Self::Light
                }
            }
            _ => Self::Dark,
        }
    }

    pub const fn colors(self) -> Colors {
        match self {
            Self::Dark => Colors {
                background: Color32::from_rgb(0x09, 0x09, 0x0B),
                surface: Color32::from_rgb(0x11, 0x11, 0x13),
                surface_secondary: Color32::from_rgb(0x18, 0x18, 0x1B),
                elevated: Color32::from_rgb(0x20, 0x20, 0x24),
                border: Color32::from_rgb(0x30, 0x30, 0x36),
                text: Color32::from_rgb(0xF4, 0xF4, 0xF5),
                text_secondary: Color32::from_rgb(0xA1, 0xA1, 0xAA),
                text_muted: Color32::from_rgb(0x71, 0x71, 0x7A),
                highlight: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                disabled: Color32::from_rgb(0x52, 0x52, 0x5B),
            },
            Self::Light => Colors {
                background: Color32::from_rgb(0xF4, 0xF4, 0xF5),
                surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                surface_secondary: Color32::from_rgb(0xEE, 0xEE, 0xF0),
                elevated: Color32::from_rgb(0xE4, 0xE4, 0xE7),
                border: Color32::from_rgb(0xD4, 0xD4, 0xD8),
                text: Color32::from_rgb(0x18, 0x18, 0x1B),
                text_secondary: Color32::from_rgb(0x52, 0x52, 0x5B),
                text_muted: Color32::from_rgb(0xA1, 0xA1, 0xAA),
                highlight: Color32::from_rgb(0x00, 0x00, 0x00),
                disabled: Color32::from_rgb(0xD4, 0xD4, 0xD8),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub background: Color32,
    pub surface: Color32,
    pub surface_secondary: Color32,
    pub elevated: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub highlight: Color32,
    pub disabled: Color32,
}

/// Apply the NOIR palette to the egui context. Called on startup and whenever
/// an appearance setting changes.
pub fn apply(ctx: &Context, palette: Palette, appearance: &AppearanceSettings, base_ppp: f32) {
    let c = palette.colors();
    let mut style = (*ctx.style_of(ctx.theme())).clone();
    let visuals = &mut style.visuals;

    *visuals = Visuals::dark();
    visuals.dark_mode = palette == Palette::Dark;
    visuals.override_text_color = Some(c.text);
    visuals.panel_fill = c.background;
    visuals.window_fill = c.surface;
    visuals.extreme_bg_color = c.background;
    visuals.faint_bg_color = c.surface_secondary;
    visuals.window_stroke = Stroke::new(1.0, c.border);
    visuals.window_corner_radius = CornerRadius::same(appearance.corner_radius.max(0.0) as u8);
    visuals.menu_corner_radius = CornerRadius::same(4);

    let widget_style = |bg: Color32, fg: Color32, stroke: Color32| egui::style::WidgetVisuals {
        bg_fill: bg,
        weak_bg_fill: bg,
        bg_stroke: Stroke::new(1.0, stroke),
        fg_stroke: Stroke::new(1.0, fg),
        corner_radius: CornerRadius::same(appearance.corner_radius.clamp(0.0, 8.0) as u8),
        expansion: 0.0,
    };
    visuals.widgets.noninteractive = widget_style(c.surface, c.text, c.border);
    visuals.widgets.inactive = widget_style(c.surface_secondary, c.text, c.border);
    visuals.widgets.hovered = widget_style(c.elevated, c.highlight, c.border);
    visuals.widgets.active = widget_style(c.elevated, c.highlight, c.text_muted);
    visuals.widgets.open = widget_style(c.surface_secondary, c.text, c.border);
    visuals.selection.bg_fill = c.elevated;
    visuals.selection.stroke = Stroke::new(1.0, c.highlight);
    visuals.hyperlink_color = c.text_secondary;

    let font = appearance.font_size.max(9.0);
    style.text_styles = [
        (TextStyle::Heading, FontId::proportional(font * 1.4)),
        (TextStyle::Body, FontId::proportional(font)),
        (TextStyle::Button, FontId::proportional(font)),
        (TextStyle::Monospace, FontId::monospace(font * 0.95)),
        (TextStyle::Small, FontId::proportional(font * 0.85)),
    ]
    .into_iter()
    .collect();

    let compact = appearance.compact_mode || appearance.dense_mode;
    let spacing = &mut style.spacing;
    spacing.item_spacing = if compact {
        Vec2::new(6.0, 3.0)
    } else {
        Vec2::new(8.0, 6.0)
    };
    spacing.button_padding = if compact {
        Vec2::new(6.0, 2.0)
    } else {
        Vec2::new(8.0, 3.0)
    };
    spacing.menu_margin = Margin::same(4);
    spacing.interact_size = Vec2::new(64.0, if compact { 20.0 } else { 24.0 });

    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
    ctx.set_pixels_per_point((base_ppp * appearance.ui_scale.max(0.5)).clamp(0.5, 4.0));
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(c: Color32) -> (u8, u8, u8) {
        (c.r(), c.g(), c.b())
    }

    #[test]
    fn dark_palette_matches_specification() {
        let c = Palette::Dark.colors();
        assert_eq!(rgb(c.background), (0x09, 0x09, 0x0B));
        assert_eq!(rgb(c.surface), (0x11, 0x11, 0x13));
        assert_eq!(rgb(c.surface_secondary), (0x18, 0x18, 0x1B));
        assert_eq!(rgb(c.elevated), (0x20, 0x20, 0x24));
        assert_eq!(rgb(c.border), (0x30, 0x30, 0x36));
        assert_eq!(rgb(c.text), (0xF4, 0xF4, 0xF5));
        assert_eq!(rgb(c.text_secondary), (0xA1, 0xA1, 0xAA));
        assert_eq!(rgb(c.text_muted), (0x71, 0x71, 0x7A));
        assert_eq!(rgb(c.highlight), (0xFF, 0xFF, 0xFF));
        assert_eq!(rgb(c.disabled), (0x52, 0x52, 0x5B));
    }

    #[test]
    fn light_palette_matches_specification() {
        let c = Palette::Light.colors();
        assert_eq!(rgb(c.background), (0xF4, 0xF4, 0xF5));
        assert_eq!(rgb(c.surface), (0xFF, 0xFF, 0xFF));
        assert_eq!(rgb(c.surface_secondary), (0xEE, 0xEE, 0xF0));
        assert_eq!(rgb(c.elevated), (0xE4, 0xE4, 0xE7));
        assert_eq!(rgb(c.border), (0xD4, 0xD4, 0xD8));
        assert_eq!(rgb(c.text), (0x18, 0x18, 0x1B));
        assert_eq!(rgb(c.text_secondary), (0x52, 0x52, 0x5B));
        assert_eq!(rgb(c.text_muted), (0xA1, 0xA1, 0xAA));
    }

    #[test]
    fn theme_index_resolution() {
        assert_eq!(Palette::from_theme_index(0, false), Palette::Dark);
        assert_eq!(Palette::from_theme_index(1, false), Palette::Light);
        assert_eq!(Palette::from_theme_index(2, true), Palette::Dark);
        assert_eq!(Palette::from_theme_index(2, false), Palette::Light);
        assert_eq!(Palette::from_theme_index(9, false), Palette::Dark);
    }

    #[test]
    fn apply_runs_headless() {
        let ctx = egui::Context::default();
        let appearance = noir_settings::schema::AppearanceSettings::default();
        apply(&ctx, Palette::Dark, &appearance, 1.0);
        assert!((ctx.pixels_per_point() - 1.0).abs() < 1e-6);
    }
}

//! Category 2 — Appearance.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const THEME_OPTIONS: [&str; 3] = ["Dark", "Light", "System"];
pub const STARTUP_SCREEN_OPTIONS: [&str; 5] = ["Home", "Player", "Library", "Playlists", "History"];
pub const OSD_POSITION_OPTIONS: [&str; 4] =
    ["Top left", "Top right", "Bottom left", "Bottom right"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    /// Index into [`THEME_OPTIONS`].
    pub theme: usize,
    pub ui_scale: f32,
    pub font_size: f32,
    pub compact_mode: bool,
    pub dense_mode: bool,
    pub control_bar_size: f32,
    pub sidebar_width: f32,
    pub playlist_width: f32,
    pub window_opacity: f32,
    pub corner_radius: f32,
    pub animations: bool,
    pub animation_speed: f32,
    pub show_logo: bool,
    pub show_info_overlay: bool,
    /// Index into [`STARTUP_SCREEN_OPTIONS`].
    pub startup_screen: usize,
    pub fullscreen_ui: bool,
    pub osd_visible: bool,
    /// Index into [`OSD_POSITION_OPTIONS`].
    pub osd_position: usize,
    pub osd_duration: f32,
    pub show_toolbar: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: 0,
            ui_scale: 1.0,
            font_size: 13.0,
            compact_mode: false,
            dense_mode: false,
            control_bar_size: 1.0,
            sidebar_width: 190.0,
            playlist_width: 320.0,
            window_opacity: 1.0,
            corner_radius: 4.0,
            animations: true,
            animation_speed: 1.0,
            show_logo: true,
            show_info_overlay: true,
            startup_screen: 1,
            fullscreen_ui: false,
            osd_visible: true,
            osd_position: 3,
            osd_duration: 1.5,
            show_toolbar: true,
        }
    }
}

impl AppearanceSettings {
    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "appearance.theme",
                "Theme",
                "Color scheme. The monochrome dark theme is the NOIR identity.",
                SettingKind::Dropdown {
                    options: THEME_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.theme),
            ),
            SettingEntry::new(
                "appearance.ui_scale",
                "UI scale",
                "Overall interface scale factor.",
                SettingKind::Slider {
                    min: 0.75,
                    max: 2.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.ui_scale),
            )
            .restart(),
            SettingEntry::new(
                "appearance.font_size",
                "Font size",
                "Base font size in points.",
                SettingKind::Slider {
                    min: 9.0,
                    max: 20.0,
                    step: 0.5,
                    unit: "pt",
                },
                SettingValue::F32(&mut self.font_size),
            ),
            SettingEntry::new(
                "appearance.compact_mode",
                "Compact interface",
                "Reduce paddings and control sizes.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.compact_mode),
            ),
            SettingEntry::new(
                "appearance.dense_mode",
                "Dense interface",
                "Increase information density in lists.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.dense_mode),
            ),
            SettingEntry::new(
                "appearance.control_bar_size",
                "Control bar size",
                "Scale of the playback control panel.",
                SettingKind::Slider {
                    min: 0.8,
                    max: 1.4,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.control_bar_size),
            ),
            SettingEntry::new(
                "appearance.sidebar_width",
                "Sidebar width",
                "Width of the navigation sidebar in pixels.",
                SettingKind::Slider {
                    min: 140.0,
                    max: 280.0,
                    step: 5.0,
                    unit: "px",
                },
                SettingValue::F32(&mut self.sidebar_width),
            ),
            SettingEntry::new(
                "appearance.playlist_width",
                "Playlist width",
                "Width of the playlist panel in pixels.",
                SettingKind::Slider {
                    min: 220.0,
                    max: 520.0,
                    step: 10.0,
                    unit: "px",
                },
                SettingValue::F32(&mut self.playlist_width),
            ),
            SettingEntry::new(
                "appearance.window_opacity",
                "Window opacity",
                "Main window opacity where supported by the platform.",
                SettingKind::Slider {
                    min: 0.5,
                    max: 1.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.window_opacity),
            ),
            SettingEntry::new(
                "appearance.corner_radius",
                "Rounded corner intensity",
                "Corner rounding of widgets in pixels.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 12.0,
                    step: 1.0,
                    unit: "px",
                },
                SettingValue::F32(&mut self.corner_radius),
            ),
            SettingEntry::new(
                "appearance.animations",
                "Enable animations",
                "Enable UI transitions and hover animations.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.animations),
            ),
            SettingEntry::new(
                "appearance.animation_speed",
                "Animation speed",
                "Multiplier for UI animation duration.",
                SettingKind::Slider {
                    min: 0.25,
                    max: 3.0,
                    step: 0.25,
                    unit: "x",
                },
                SettingValue::F32(&mut self.animation_speed),
            ),
            SettingEntry::new(
                "appearance.show_logo",
                "Show application logo",
                "Display the NOIR Player logo in the top bar.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_logo),
            ),
            SettingEntry::new(
                "appearance.show_info_overlay",
                "Show video information overlay",
                "Display technical media information over the video.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_info_overlay),
            ),
            SettingEntry::new(
                "appearance.startup_screen",
                "Default startup screen",
                "Screen shown when the application starts.",
                SettingKind::Dropdown {
                    options: STARTUP_SCREEN_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.startup_screen),
            ),
            SettingEntry::new(
                "appearance.fullscreen_ui",
                "Fullscreen interface mode",
                "Hide window chrome and use the full surface for playback.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.fullscreen_ui),
            ),
            SettingEntry::new(
                "appearance.osd_visible",
                "OSD visibility",
                "Show on-screen messages (seek, volume, speed).",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.osd_visible),
            ),
            SettingEntry::new(
                "appearance.osd_position",
                "OSD position",
                "Corner where OSD messages appear.",
                SettingKind::Dropdown {
                    options: OSD_POSITION_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.osd_position),
            ),
            SettingEntry::new(
                "appearance.osd_duration",
                "OSD duration",
                "Seconds an OSD message stays visible.",
                SettingKind::Slider {
                    min: 0.3,
                    max: 5.0,
                    step: 0.1,
                    unit: "s",
                },
                SettingValue::F32(&mut self.osd_duration),
            ),
            SettingEntry::new(
                "appearance.show_toolbar",
                "Show toolbar",
                "Display the customizable toolbar in the top bar.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_toolbar),
            ),
        ]
    }
}

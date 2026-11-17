//! Category 6 — Subtitles.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const TRACK_SELECTION_OPTIONS: [&str; 4] =
    ["Auto", "Preferred language", "First available", "None"];
pub const ASS_OVERRIDE_OPTIONS: [&str; 4] =
    ["No", "Yes (strip styles)", "Yes and default", "Force"];
pub const FORCED_SUB_OPTIONS: [&str; 3] = ["Off", "Only forced", "Prefer forced"];
pub const HI_SUB_OPTIONS: [&str; 3] =
    ["Normal", "Prefer hearing impaired", "Only hearing impaired"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SubtitleSettings {
    pub autoload_external: bool,
    pub preferred_languages: String,
    /// Index into [`TRACK_SELECTION_OPTIONS`].
    pub track_selection: usize,
    /// Dynamic subtitle delay in seconds (mpv `sub-delay`).
    pub sub_delay: f32,
    /// Subtitle playback speed ratio (mpv `sub-speed`).
    pub sub_speed: f32,
    pub font: String,
    pub font_size: u32,
    /// Index into weight options: normal/bold/light.
    pub font_weight: usize,
    /// Font color as `#RRGGBB` or `#AARRGGBB`.
    pub font_color: String,
    pub outline_color: String,
    pub outline_size: f32,
    pub shadow_size: f32,
    /// Vertical position as percentage from bottom, 0..=100.
    pub vertical_position: f32,
    /// Index into align options: center/left/right.
    pub horizontal_align: usize,
    pub margin_v: u32,
    pub margin_h: u32,
    /// Background box opacity 0..=1.
    pub background_opacity: f32,
    /// Index into [`ASS_OVERRIDE_OPTIONS`].
    pub ass_override: usize,
    /// Index into [`FORCED_SUB_OPTIONS`].
    pub forced_subtitles: usize,
    /// Index into [`HI_SUB_OPTIONS`].
    pub hearing_impaired: usize,
    /// Text encoding for external subtitles, e.g. "UTF-8", "auto".
    pub encoding: String,
    pub external_loading: bool,
    pub visible: bool,
}

impl Default for SubtitleSettings {
    fn default() -> Self {
        Self {
            autoload_external: true,
            preferred_languages: String::new(),
            track_selection: 0,
            sub_delay: 0.0,
            sub_speed: 1.0,
            font: String::new(),
            font_size: 55,
            font_weight: 0,
            font_color: "#FFFFFFFF".into(),
            outline_color: "#FF000000".into(),
            outline_size: 3.0,
            shadow_size: 0.0,
            vertical_position: 100.0,
            horizontal_align: 0,
            margin_v: 22,
            margin_h: 0,
            background_opacity: 0.0,
            ass_override: 0,
            forced_subtitles: 0,
            hearing_impaired: 0,
            encoding: "auto".into(),
            external_loading: true,
            visible: true,
        }
    }
}

impl SubtitleSettings {
    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "subtitles.autoload_external",
                "Automatically load external subtitles",
                "Load .srt/.ass files sitting next to the video.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.autoload_external),
            ),
            SettingEntry::new(
                "subtitles.preferred_languages",
                "Preferred subtitle languages",
                "ISO codes in priority order, e.g. vie,eng.",
                SettingKind::Text,
                SettingValue::Text(&mut self.preferred_languages),
            ),
            SettingEntry::new(
                "subtitles.track_selection",
                "Subtitle track selection",
                "How a subtitle track is chosen when media opens.",
                SettingKind::Dropdown {
                    options: TRACK_SELECTION_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.track_selection),
            ),
            SettingEntry::new(
                "subtitles.sub_delay",
                "Subtitle synchronization",
                "Subtitle delay in seconds.",
                SettingKind::Slider {
                    min: -30.0,
                    max: 30.0,
                    step: 0.1,
                    unit: "s",
                },
                SettingValue::F32(&mut self.sub_delay),
            ),
            SettingEntry::new(
                "subtitles.sub_speed",
                "Subtitle speed",
                "Playback speed ratio for subtitles.",
                SettingKind::Slider {
                    min: 0.5,
                    max: 2.0,
                    step: 0.01,
                    unit: "x",
                },
                SettingValue::F32(&mut self.sub_speed),
            ),
            SettingEntry::new(
                "subtitles.font",
                "Subtitle font",
                "Font family used for plain text subtitles.",
                SettingKind::Text,
                SettingValue::Text(&mut self.font),
            ),
            SettingEntry::new(
                "subtitles.font_size",
                "Font size",
                "Subtitle font size in script units.",
                SettingKind::Slider {
                    min: 10.0,
                    max: 120.0,
                    step: 1.0,
                    unit: "pt",
                },
                SettingValue::U32(&mut self.font_size),
            ),
            SettingEntry::new(
                "subtitles.font_weight",
                "Font weight",
                "Weight of the subtitle font.",
                SettingKind::Dropdown {
                    options: ["Normal", "Bold", "Light"].to_vec(),
                },
                SettingValue::Choice(&mut self.font_weight),
            ),
            SettingEntry::new(
                "subtitles.font_color",
                "Font color",
                "Subtitle text color as #RRGGBB or #AARRGGBB.",
                SettingKind::Text,
                SettingValue::Text(&mut self.font_color),
            ),
            SettingEntry::new(
                "subtitles.outline_color",
                "Outline color",
                "Subtitle outline color.",
                SettingKind::Text,
                SettingValue::Text(&mut self.outline_color),
            ),
            SettingEntry::new(
                "subtitles.outline_size",
                "Outline thickness",
                "Outline border width.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 10.0,
                    step: 0.5,
                    unit: "px",
                },
                SettingValue::F32(&mut self.outline_size),
            ),
            SettingEntry::new(
                "subtitles.shadow_size",
                "Shadow",
                "Drop shadow depth behind subtitles.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 10.0,
                    step: 0.5,
                    unit: "px",
                },
                SettingValue::F32(&mut self.shadow_size),
            ),
            SettingEntry::new(
                "subtitles.vertical_position",
                "Vertical positioning",
                "Vertical subtitle position, percent from bottom.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "%",
                },
                SettingValue::F32(&mut self.vertical_position),
            ),
            SettingEntry::new(
                "subtitles.horizontal_align",
                "Horizontal positioning",
                "Horizontal alignment of subtitle text.",
                SettingKind::Dropdown {
                    options: ["Center", "Left", "Right"].to_vec(),
                },
                SettingValue::Choice(&mut self.horizontal_align),
            ),
            SettingEntry::new(
                "subtitles.margin_v",
                "Vertical margin",
                "Bottom margin in script units.",
                SettingKind::Number {
                    min: 0.0,
                    max: 200.0,
                    step: 1.0,
                },
                SettingValue::U32(&mut self.margin_v),
            ),
            SettingEntry::new(
                "subtitles.margin_h",
                "Horizontal margin",
                "Side margin in script units.",
                SettingKind::Number {
                    min: 0.0,
                    max: 200.0,
                    step: 1.0,
                },
                SettingValue::U32(&mut self.margin_h),
            ),
            SettingEntry::new(
                "subtitles.background_opacity",
                "Background opacity",
                "Opaque box behind subtitle text, 0-100%.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 1.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.background_opacity),
            ),
            SettingEntry::new(
                "subtitles.ass_override",
                "ASS/SSA styling behavior",
                "Whether custom styling overrides embedded ASS styles.",
                SettingKind::Dropdown {
                    options: ASS_OVERRIDE_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.ass_override),
            ),
            SettingEntry::new(
                "subtitles.forced_subtitles",
                "Forced subtitles",
                "Prefer forced subtitle tracks (dialogue in foreign scenes).",
                SettingKind::Dropdown {
                    options: FORCED_SUB_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.forced_subtitles),
            ),
            SettingEntry::new(
                "subtitles.hearing_impaired",
                "Hearing-impaired subtitles",
                "Preference for SDH tracks.",
                SettingKind::Dropdown {
                    options: HI_SUB_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.hearing_impaired),
            ),
            SettingEntry::new(
                "subtitles.encoding",
                "Subtitle encoding",
                "Text encoding for external subtitles.",
                SettingKind::Text,
                SettingValue::Text(&mut self.encoding),
            ),
            SettingEntry::new(
                "subtitles.external_loading",
                "External subtitle loading",
                "Allow loading subtitle files manually.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.external_loading),
            ),
            SettingEntry::new(
                "subtitles.visible",
                "Subtitle visibility",
                "Show subtitles when a track is selected.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.visible),
            ),
        ]
    }
}

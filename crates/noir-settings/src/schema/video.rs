//! Category 4 — Video. Options here map to real mpv properties; entries that
//! the active backend cannot honor are hidden by the capability gate.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const RENDERER_OPTIONS: [&str; 3] = ["Auto", "gpu-next", "gpu"];
pub const HWDEC_OPTIONS: [&str; 7] = [
    "Auto", "Disabled", "d3d11va", "dxva2", "nvdec", "vaapi", "vdpau",
];
pub const VIDEO_SYNC_OPTIONS: [&str; 6] = [
    "audio",
    "display-resample",
    "display-resample-vdrop",
    "display-vdrop",
    "desync",
    "audio-first",
];
pub const DEINTERLACE_OPTIONS: [&str; 4] = ["Auto", "No", "Yes", "Adaptive"];
pub const ASPECT_OPTIONS: [&str; 6] = ["Auto", "16:9", "4:3", "2.35:1", "1:1", "Custom"];
pub const ROTATION_OPTIONS: [&str; 4] = ["0°", "90°", "180°", "270°"];
pub const SCALE_OPTIONS: [&str; 6] = [
    "bilinear",
    "spline36",
    "lanczos",
    "ewa_lanczos",
    "mitchell",
    "bicubic",
];
pub const TONEMAP_OPTIONS: [&str; 5] = ["auto", "bt.2446a", "spline", "bt.2390", "clip"];
pub const SCREENSHOT_FORMATS: [&str; 3] = ["png", "jpeg", "webp"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VideoSettings {
    /// Index into [`RENDERER_OPTIONS`].
    pub renderer: usize,
    /// Index into [`HWDEC_OPTIONS`].
    pub hwdec: usize,
    pub hwdec_automatic: bool,
    pub decoder_threads: u32,
    /// Index into [`VIDEO_SYNC_OPTIONS`].
    pub video_sync: usize,
    pub display_sync: bool,
    pub interpolation: bool,
    /// Index into [`DEINTERLACE_OPTIONS`].
    pub deinterlace: usize,
    /// Index into [`ASPECT_OPTIONS`].
    pub aspect_ratio: usize,
    pub aspect_custom: String,
    /// Index into crop presets: none/16:9/4:3/2.35:1.
    pub crop_mode: usize,
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    /// Index into [`ROTATION_OPTIONS`].
    pub rotation: usize,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub gamma: i32,
    pub hue: i32,
    /// Index into [`SCALE_OPTIONS`].
    pub scale_algorithm: usize,
    pub chroma_scale_algorithm: usize,
    pub dithering: bool,
    pub debanding: bool,
    pub hdr_passthrough: bool,
    /// Index into [`TONEMAP_OPTIONS`].
    pub hdr_tone_mapping: usize,
    pub color_management: bool,
    /// Index into [`SCREENSHOT_FORMATS`].
    pub screenshot_format: usize,
    pub screenshot_quality: u32,
    pub screenshot_template: String,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            renderer: 0,
            hwdec: 0,
            hwdec_automatic: true,
            decoder_threads: 0,
            video_sync: 0,
            display_sync: false,
            interpolation: false,
            deinterlace: 0,
            aspect_ratio: 0,
            aspect_custom: String::new(),
            crop_mode: 0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0,
            flip_horizontal: false,
            flip_vertical: false,
            brightness: 0,
            contrast: 0,
            saturation: 0,
            gamma: 0,
            hue: 0,
            scale_algorithm: 1,
            chroma_scale_algorithm: 1,
            dithering: true,
            debanding: false,
            hdr_passthrough: false,
            hdr_tone_mapping: 0,
            color_management: true,
            screenshot_format: 0,
            screenshot_quality: 90,
            screenshot_template: "%F_%n_%t".into(),
        }
    }
}

impl VideoSettings {
    /// mpv `hwdec` value for the selected option.
    pub fn hwdec_value(&self) -> &'static str {
        match self.hwdec {
            0 => "auto-safe",
            1 => "no",
            2 => "d3d11va",
            3 => "dxva2",
            4 => "nvdec",
            5 => "vaapi",
            6 => "vdpau",
            _ => "auto-safe",
        }
    }

    pub fn video_sync_value(&self) -> &'static str {
        VIDEO_SYNC_OPTIONS
            .get(self.video_sync)
            .copied()
            .unwrap_or("audio")
    }

    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "video.renderer",
                "Video renderer",
                "mpv render backend. gpu-next is the modern path.",
                SettingKind::Dropdown {
                    options: RENDERER_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.renderer),
            )
            .restart(),
            SettingEntry::new(
                "video.hwdec",
                "Hardware decoding",
                "Hardware-accelerated decoder family.",
                SettingKind::Dropdown {
                    options: HWDEC_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.hwdec),
            ),
            SettingEntry::new(
                "video.hwdec_automatic",
                "Automatic decoder selection",
                "Let the backend pick the best available hardware decoder.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.hwdec_automatic),
            ),
            SettingEntry::new(
                "video.decoder_threads",
                "Decoder threads",
                "Decoding threads. 0 = automatic.",
                SettingKind::Number {
                    min: 0.0,
                    max: 32.0,
                    step: 1.0,
                },
                SettingValue::U32(&mut self.decoder_threads),
            ),
            SettingEntry::new(
                "video.video_sync",
                "Video synchronization",
                "A/V synchronization mode.",
                SettingKind::Dropdown {
                    options: VIDEO_SYNC_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.video_sync),
            ),
            SettingEntry::new(
                "video.display_sync",
                "Display synchronization",
                "Resample audio to lock video to the display refresh rate.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.display_sync),
            ),
            SettingEntry::new(
                "video.interpolation",
                "Frame interpolation",
                "Interpolate intermediate frames (higher GPU cost).",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.interpolation),
            ),
            SettingEntry::new(
                "video.deinterlace",
                "Deinterlacing",
                "Deinterlace interlaced sources.",
                SettingKind::Dropdown {
                    options: DEINTERLACE_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.deinterlace),
            ),
            SettingEntry::new(
                "video.aspect_ratio",
                "Aspect ratio",
                "Display aspect ratio override.",
                SettingKind::Dropdown {
                    options: ASPECT_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.aspect_ratio),
            ),
            SettingEntry::new(
                "video.aspect_custom",
                "Custom aspect ratio",
                "Used when aspect ratio is Custom, e.g. 2.0:1.",
                SettingKind::Text,
                SettingValue::Text(&mut self.aspect_custom),
            ),
            SettingEntry::new(
                "video.crop_mode",
                "Crop mode",
                "Crop the video to a preset ratio.",
                SettingKind::Dropdown {
                    options: ["None", "16:9", "4:3", "2.35:1"].to_vec(),
                },
                SettingValue::Choice(&mut self.crop_mode),
            ),
            SettingEntry::new(
                "video.zoom",
                "Zoom",
                "Video zoom factor.",
                SettingKind::Slider {
                    min: 0.5,
                    max: 4.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.zoom),
            ),
            SettingEntry::new(
                "video.pan_x",
                "Pan horizontal",
                "Horizontal pan offset.",
                SettingKind::Slider {
                    min: -1.0,
                    max: 1.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.pan_x),
            ),
            SettingEntry::new(
                "video.pan_y",
                "Pan vertical",
                "Vertical pan offset.",
                SettingKind::Slider {
                    min: -1.0,
                    max: 1.0,
                    step: 0.05,
                    unit: "x",
                },
                SettingValue::F32(&mut self.pan_y),
            ),
            SettingEntry::new(
                "video.rotation",
                "Rotation",
                "Rotate the video.",
                SettingKind::Dropdown {
                    options: ROTATION_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.rotation),
            ),
            SettingEntry::new(
                "video.flip_horizontal",
                "Flip horizontal",
                "Mirror the video horizontally.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.flip_horizontal),
            ),
            SettingEntry::new(
                "video.flip_vertical",
                "Flip vertical",
                "Mirror the video vertically.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.flip_vertical),
            ),
            SettingEntry::new(
                "video.brightness",
                "Brightness",
                "Brightness adjustment.",
                SettingKind::Slider {
                    min: -100.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "",
                },
                SettingValue::I32(&mut self.brightness),
            ),
            SettingEntry::new(
                "video.contrast",
                "Contrast",
                "Contrast adjustment.",
                SettingKind::Slider {
                    min: -100.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "",
                },
                SettingValue::I32(&mut self.contrast),
            ),
            SettingEntry::new(
                "video.saturation",
                "Saturation",
                "Saturation adjustment.",
                SettingKind::Slider {
                    min: -100.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "",
                },
                SettingValue::I32(&mut self.saturation),
            ),
            SettingEntry::new(
                "video.gamma",
                "Gamma",
                "Gamma adjustment.",
                SettingKind::Slider {
                    min: -100.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "",
                },
                SettingValue::I32(&mut self.gamma),
            ),
            SettingEntry::new(
                "video.hue",
                "Hue",
                "Hue adjustment in degrees.",
                SettingKind::Slider {
                    min: -180.0,
                    max: 180.0,
                    step: 1.0,
                    unit: "°",
                },
                SettingValue::I32(&mut self.hue),
            ),
            SettingEntry::new(
                "video.scale_algorithm",
                "Video scaling algorithm",
                "Luma upscaling filter.",
                SettingKind::Dropdown {
                    options: SCALE_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.scale_algorithm),
            ),
            SettingEntry::new(
                "video.chroma_scale_algorithm",
                "Chroma scaling algorithm",
                "Chroma upscaling filter.",
                SettingKind::Dropdown {
                    options: SCALE_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.chroma_scale_algorithm),
            ),
            SettingEntry::new(
                "video.dithering",
                "Dithering",
                "Dither to reduce color banding on low-depth displays.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.dithering),
            ),
            SettingEntry::new(
                "video.debanding",
                "Debanding",
                "Remove banding artifacts from compressed sources.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.debanding),
            ),
            SettingEntry::new(
                "video.hdr_passthrough",
                "HDR passthrough",
                "Pass HDR metadata to the display when supported.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.hdr_passthrough),
            )
            .requires("hdr"),
            SettingEntry::new(
                "video.hdr_tone_mapping",
                "HDR tone mapping",
                "Algorithm used to map HDR to SDR.",
                SettingKind::Dropdown {
                    options: TONEMAP_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.hdr_tone_mapping),
            ),
            SettingEntry::new(
                "video.color_management",
                "Color management",
                "Respect embedded color metadata.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.color_management),
            ),
            SettingEntry::new(
                "video.screenshot_format",
                "Screenshot format",
                "Image format for screenshots.",
                SettingKind::Dropdown {
                    options: SCREENSHOT_FORMATS.to_vec(),
                },
                SettingValue::Choice(&mut self.screenshot_format),
            ),
            SettingEntry::new(
                "video.screenshot_quality",
                "Screenshot quality",
                "Compression quality for lossy screenshot formats.",
                SettingKind::Slider {
                    min: 50.0,
                    max: 100.0,
                    step: 1.0,
                    unit: "%",
                },
                SettingValue::U32(&mut self.screenshot_quality),
            ),
            SettingEntry::new(
                "video.screenshot_template",
                "Screenshot filename template",
                "mpv screenshot template, e.g. %F_%n_%t.",
                SettingKind::Text,
                SettingValue::Text(&mut self.screenshot_template),
            ),
        ]
    }
}

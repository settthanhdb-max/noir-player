//! Category 5 — Audio.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const AO_BACKENDS: [&str; 5] = ["Auto", "wasapi", "directsound", "winmm", "null"];
pub const CHANNEL_LAYOUTS: [&str; 6] =
    ["Auto", "Stereo", "5.1", "7.1", "Mono", "Downmix to stereo"];
pub const RESAMPLE_QUALITIES: [&str; 4] = ["Fast", "Medium", "High", "Very high"];
pub const REPLAYGAIN_OPTIONS: [&str; 4] = ["Disabled", "Track", "Album", "Preamp only"];
pub const STEREO_MODES: [&str; 4] = ["Auto", "Stereo", "Mono", "Reverse stereo"];
pub const SAMPLE_RATES: [&str; 5] = ["Auto", "44100", "48000", "88200", "96000"];
pub const OUTPUT_FORMATS: [&str; 5] = ["Auto", "s16", "s24", "s32", "float"];
/// Center frequencies of the built-in 10-band equalizer, in Hz.
pub const EQ_BAND_HZ: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];
pub const EQ_KEYS: [&str; 10] = [
    "audio.eq_band_31hz",
    "audio.eq_band_62hz",
    "audio.eq_band_125hz",
    "audio.eq_band_250hz",
    "audio.eq_band_500hz",
    "audio.eq_band_1khz",
    "audio.eq_band_2khz",
    "audio.eq_band_4khz",
    "audio.eq_band_8khz",
    "audio.eq_band_16khz",
];
pub const EQ_LABELS: [&str; 10] = [
    "EQ 31 Hz",
    "EQ 62 Hz",
    "EQ 125 Hz",
    "EQ 250 Hz",
    "EQ 500 Hz",
    "EQ 1 kHz",
    "EQ 2 kHz",
    "EQ 4 kHz",
    "EQ 8 kHz",
    "EQ 16 kHz",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Index into [`AO_BACKENDS`].
    pub ao_backend: usize,
    pub device: String,
    /// Index into [`CHANNEL_LAYOUTS`].
    pub channel_layout: usize,
    /// A/V sync offset in milliseconds.
    pub audio_sync_ms: f32,
    /// Static audio delay in seconds (mpv `audio-delay`).
    pub audio_delay: f32,
    pub resampling: bool,
    /// Index into [`RESAMPLE_QUALITIES`].
    pub resample_quality: usize,
    pub volume_normalization: bool,
    /// Index into [`REPLAYGAIN_OPTIONS`].
    pub replaygain: usize,
    /// Extra amplification in dB (mpv `volume`), up to +20.
    pub amplification_db: f32,
    pub equalizer_enabled: bool,
    /// Gains in dB for the 10-band equalizer, -12..=12.
    pub eq_gains: Vec<f32>,
    /// Index into EQ presets: flat/bass/treble/vocal/rock.
    pub eq_preset: usize,
    pub downmix: bool,
    /// Index into [`STEREO_MODES`].
    pub stereo_mode: usize,
    /// Index into [`SAMPLE_RATES`].
    pub sample_rate: usize,
    /// Index into [`OUTPUT_FORMATS`].
    pub output_format: usize,
    pub passthrough: bool,
    pub auto_select_track: bool,
    pub preferred_language: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            ao_backend: 0,
            device: String::new(),
            channel_layout: 0,
            audio_sync_ms: 0.0,
            audio_delay: 0.0,
            resampling: true,
            resample_quality: 2,
            volume_normalization: false,
            replaygain: 0,
            amplification_db: 0.0,
            equalizer_enabled: false,
            eq_gains: vec![0.0; 10],
            eq_preset: 0,
            downmix: false,
            stereo_mode: 0,
            sample_rate: 0,
            output_format: 0,
            passthrough: false,
            auto_select_track: true,
            preferred_language: String::new(),
        }
    }
}

impl AudioSettings {
    /// mpv volume is dB-based; 100 == 0 dB. Amplification shifts the base.
    pub fn amplification_to_volume(&self) -> u32 {
        (100.0 * 10f32.powf(self.amplification_db / 20.0)).clamp(0.0, 1000.0) as u32
    }

    /// Build the mpv `af` lavfi chain implementing the equalizer and
    /// normalization. Returns `None` when no filter is active.
    pub fn audio_filter_chain(&self) -> Option<String> {
        let mut filters: Vec<String> = Vec::new();
        if self.equalizer_enabled {
            let gains: Vec<String> = self.eq_gains.iter().map(|g| format!("{g:.1}")).collect();
            filters.push(format!(
                "lavfi=[superequalizer=1={}:2={}:3={}:4={}:5={}:6={}:7={}:8={}:9={}:10={}]",
                gains[0],
                gains[1],
                gains[2],
                gains[3],
                gains[4],
                gains[5],
                gains[6],
                gains[7],
                gains[8],
                gains[9]
            ));
        }
        if self.volume_normalization {
            filters.push("lavfi=[loudnorm]".into());
        }
        if self.downmix {
            filters.push("lavfi=[pan=stereo|c0=c0+c2+c3|c1=c1+c2+c3]".into());
        }
        if filters.is_empty() {
            None
        } else {
            Some(filters.join(","))
        }
    }

    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        let mut entries = vec![
            SettingEntry::new(
                "audio.ao_backend",
                "Audio output backend",
                "Audio output implementation used by the engine.",
                SettingKind::Dropdown {
                    options: AO_BACKENDS.to_vec(),
                },
                SettingValue::Choice(&mut self.ao_backend),
            )
            .restart(),
            SettingEntry::new(
                "audio.device",
                "Output device",
                "Audio device identifier. Empty selects the system default.",
                SettingKind::Text,
                SettingValue::Text(&mut self.device),
            ),
            SettingEntry::new(
                "audio.channel_layout",
                "Output channel layout",
                "Speaker layout requested from the device.",
                SettingKind::Dropdown {
                    options: CHANNEL_LAYOUTS.to_vec(),
                },
                SettingValue::Choice(&mut self.channel_layout),
            ),
            SettingEntry::new(
                "audio.audio_sync_ms",
                "Audio synchronization",
                "A/V sync offset in milliseconds.",
                SettingKind::Slider {
                    min: -500.0,
                    max: 500.0,
                    step: 5.0,
                    unit: "ms",
                },
                SettingValue::F32(&mut self.audio_sync_ms),
            ),
            SettingEntry::new(
                "audio.audio_delay",
                "Audio delay",
                "Static audio delay in seconds.",
                SettingKind::Slider {
                    min: -10.0,
                    max: 10.0,
                    step: 0.05,
                    unit: "s",
                },
                SettingValue::F32(&mut self.audio_delay),
            ),
            SettingEntry::new(
                "audio.resampling",
                "Audio resampling",
                "Resample audio to the output device rate.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.resampling),
            ),
            SettingEntry::new(
                "audio.resample_quality",
                "Resampling quality",
                "Resampler quality versus CPU cost.",
                SettingKind::Dropdown {
                    options: RESAMPLE_QUALITIES.to_vec(),
                },
                SettingValue::Choice(&mut self.resample_quality),
            ),
            SettingEntry::new(
                "audio.volume_normalization",
                "Volume normalization",
                "Normalize loudness across files.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.volume_normalization),
            ),
            SettingEntry::new(
                "audio.replaygain",
                "ReplayGain",
                "Apply ReplayGain tags where present.",
                SettingKind::Dropdown {
                    options: REPLAYGAIN_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.replaygain),
            ),
            SettingEntry::new(
                "audio.amplification_db",
                "Audio amplification",
                "Software volume boost in decibels.",
                SettingKind::Slider {
                    min: -60.0,
                    max: 20.0,
                    step: 0.5,
                    unit: "dB",
                },
                SettingValue::F32(&mut self.amplification_db),
            ),
            SettingEntry::new(
                "audio.equalizer_enabled",
                "Equalizer",
                "Enable the 10-band equalizer.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.equalizer_enabled),
            ),
            SettingEntry::new(
                "audio.eq_preset",
                "Audio preset",
                "Equalizer preset.",
                SettingKind::Dropdown {
                    options: ["Flat", "Bass boost", "Treble boost", "Vocal", "Rock"].to_vec(),
                },
                SettingValue::Choice(&mut self.eq_preset),
            ),
            SettingEntry::new(
                "audio.downmix",
                "Channel downmixing",
                "Downmix multichannel audio to stereo.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.downmix),
            ),
            SettingEntry::new(
                "audio.stereo_mode",
                "Stereo mode",
                "Stereo playback mode.",
                SettingKind::Dropdown {
                    options: STEREO_MODES.to_vec(),
                },
                SettingValue::Choice(&mut self.stereo_mode),
            ),
            SettingEntry::new(
                "audio.sample_rate",
                "Output sample rate",
                "Force a specific output sample rate.",
                SettingKind::Dropdown {
                    options: SAMPLE_RATES.to_vec(),
                },
                SettingValue::Choice(&mut self.sample_rate),
            ),
            SettingEntry::new(
                "audio.output_format",
                "Output format",
                "Sample format requested from the device.",
                SettingKind::Dropdown {
                    options: OUTPUT_FORMATS.to_vec(),
                },
                SettingValue::Choice(&mut self.output_format),
            ),
            SettingEntry::new(
                "audio.passthrough",
                "Audio passthrough",
                "Bitstream passthrough to an AV receiver.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.passthrough),
            )
            .requires("passthrough"),
            SettingEntry::new(
                "audio.auto_select_track",
                "Automatically select audio track",
                "Pick the best audio track when a file opens.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_select_track),
            ),
            SettingEntry::new(
                "audio.preferred_language",
                "Preferred audio language",
                "ISO language codes in priority order, e.g. vie,eng.",
                SettingKind::Text,
                SettingValue::Text(&mut self.preferred_language),
            ),
        ];
        let gains = self.eq_gains.as_mut_slice();
        for (i, gain) in gains.iter_mut().enumerate() {
            entries.push(
                SettingEntry::new(
                    EQ_KEYS[i],
                    EQ_LABELS[i],
                    "Equalizer band gain in decibels.",
                    SettingKind::Slider {
                        min: -12.0,
                        max: 12.0,
                        step: 0.5,
                        unit: "dB",
                    },
                    SettingValue::F32(gain),
                )
                .advanced(),
            );
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_chain_only_when_active() {
        let a = AudioSettings::default();
        assert_eq!(a.audio_filter_chain(), None);
        let mut a = AudioSettings {
            equalizer_enabled: true,
            ..Default::default()
        };
        assert!(a.audio_filter_chain().unwrap().contains("superequalizer"));
        a.volume_normalization = true;
        let chain = a.audio_filter_chain().unwrap();
        assert!(chain.contains("loudnorm") && chain.contains(','));
    }
}

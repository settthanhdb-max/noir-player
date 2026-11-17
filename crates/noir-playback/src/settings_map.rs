//! Settings -> engine command mapping.
//!
//! This is the only place that knows how typed settings translate into
//! engine operations, keeping both the settings schema and the backends
//! independent of each other.

use crate::engine::EngineCommand;
use noir_settings::schema::{
    AudioSettings, NetworkSettings, PlaybackSettings, SubtitleSettings, VideoSettings,
};

/// Commands to apply when the engine starts or settings change wholesale.
pub fn initial_commands(
    video: &VideoSettings,
    audio: &AudioSettings,
    subtitles: &SubtitleSettings,
    network: &NetworkSettings,
    playback: &PlaybackSettings,
) -> Vec<EngineCommand> {
    let mut cmds = vec![EngineCommand::SetHwdec {
        mode: video.hwdec_value().into(),
    }];
    cmds.push(EngineCommand::SetVideoEqualizer {
        brightness: video.brightness,
        contrast: video.contrast,
        saturation: video.saturation,
        gamma: video.gamma,
        hue: video.hue,
    });
    cmds.push(EngineCommand::SetZoomPan {
        zoom: video.zoom as f64,
        pan_x: video.pan_x as f64,
        pan_y: video.pan_y as f64,
    });
    cmds.push(EngineCommand::SetRotation {
        degrees: (video.rotation as u32) * 90,
    });
    cmds.push(EngineCommand::SetFlip {
        horizontal: video.flip_horizontal,
        vertical: video.flip_vertical,
    });
    cmds.push(EngineCommand::SetDeinterlace {
        mode: match video.deinterlace {
            0 => "auto".into(),
            1 => "no".into(),
            2 => "yes".into(),
            _ => "auto".into(),
        },
    });
    cmds.push(EngineCommand::SetAudioFilters {
        chain: audio.audio_filter_chain(),
    });
    cmds.push(EngineCommand::SetVolume {
        volume: audio.amplification_to_volume(),
    });
    cmds.push(EngineCommand::SetSubDelay {
        seconds: subtitles.sub_delay as f64,
    });
    cmds.push(EngineCommand::SetSubVisibility {
        visible: subtitles.visible,
    });
    cmds.push(EngineCommand::SetSpeed {
        speed: playback.default_speed as f64,
    });
    cmds.push(EngineCommand::ApplyRawOptions {
        options: network.raw_options(),
    });
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_commands_cover_core_properties() {
        let cmds = initial_commands(
            &VideoSettings::default(),
            &AudioSettings::default(),
            &SubtitleSettings::default(),
            &NetworkSettings::default(),
            &PlaybackSettings::default(),
        );
        assert!(cmds
            .iter()
            .any(|c| matches!(c, EngineCommand::SetHwdec { .. })));
        assert!(cmds
            .iter()
            .any(|c| matches!(c, EngineCommand::SetAudioFilters { .. })));
        assert!(cmds
            .iter()
            .any(|c| matches!(c, EngineCommand::ApplyRawOptions { .. })));
    }

    #[test]
    fn amplification_converts_decibels() {
        let a = AudioSettings {
            amplification_db: 0.0,
            ..Default::default()
        };
        assert_eq!(a.amplification_to_volume(), 100);
        let a = AudioSettings {
            amplification_db: 6.0,
            ..Default::default()
        };
        assert!((a.amplification_to_volume() as i32 - 200).abs() < 3);
    }

    #[test]
    fn network_options_map_to_mpv() {
        let n = NetworkSettings::default();
        let opts = n.raw_options();
        assert!(opts.contains(&("network-timeout".into(), "30".into())));
        assert!(opts.iter().any(|(k, _)| k == "demuxer-max-bytes"));
    }
}

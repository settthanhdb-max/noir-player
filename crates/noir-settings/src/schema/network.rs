//! Category 7 — Network.

use crate::value::{SettingEntry, SettingKind, SettingValue};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_OPTIONS: [&str; 4] = ["All", "HTTP(S) only", "HTTP + HLS", "LAN only"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    /// Network operation timeout in seconds.
    pub timeout_s: u32,
    /// Demuxer cache size in MiB.
    pub cache_size_mib: u32,
    /// Initial buffering in seconds before playback starts.
    pub initial_buffering_s: u32,
    pub auto_reconnect: bool,
    pub retry_count: u32,
    /// Backbuffer for streams in MiB.
    pub stream_buffer_mib: u32,
    /// Index into [`PROTOCOL_OPTIONS`].
    pub protocols: usize,
    pub max_concurrent: u32,
    pub user_agent: String,
    pub show_status: bool,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            timeout_s: 30,
            cache_size_mib: 64,
            initial_buffering_s: 2,
            auto_reconnect: true,
            retry_count: 3,
            stream_buffer_mib: 32,
            protocols: 0,
            max_concurrent: 4,
            user_agent: "NOIRPlayer/0.1".into(),
            show_status: true,
        }
    }
}

impl NetworkSettings {
    /// mpv option pairs implementing the network category.
    pub fn raw_options(&self) -> Vec<(String, String)> {
        vec![
            ("network-timeout".into(), self.timeout_s.to_string()),
            (
                "demuxer-max-bytes".into(),
                format!("{}MiB", self.cache_size_mib),
            ),
            (
                "demuxer-readahead-secs".into(),
                self.initial_buffering_s.to_string(),
            ),
            (
                "stream-buffer-size".into(),
                format!("{}MiB", self.stream_buffer_mib),
            ),
            ("user-agent".into(), self.user_agent.clone()),
        ]
    }

    pub fn describe(&mut self) -> Vec<SettingEntry<'_>> {
        vec![
            SettingEntry::new(
                "network.timeout_s",
                "Network timeout",
                "Seconds before a stalled network operation fails.",
                SettingKind::Slider {
                    min: 5.0,
                    max: 300.0,
                    step: 5.0,
                    unit: "s",
                },
                SettingValue::U32(&mut self.timeout_s),
            ),
            SettingEntry::new(
                "network.cache_size_mib",
                "Network cache size",
                "Demuxer cache in mebibytes.",
                SettingKind::Slider {
                    min: 4.0,
                    max: 1024.0,
                    step: 4.0,
                    unit: "MiB",
                },
                SettingValue::U32(&mut self.cache_size_mib),
            ),
            SettingEntry::new(
                "network.initial_buffering_s",
                "Initial buffering",
                "Seconds of media buffered before playback starts.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 30.0,
                    step: 1.0,
                    unit: "s",
                },
                SettingValue::U32(&mut self.initial_buffering_s),
            ),
            SettingEntry::new(
                "network.auto_reconnect",
                "Reconnection behavior",
                "Automatically reconnect after a network interruption.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.auto_reconnect),
            ),
            SettingEntry::new(
                "network.retry_count",
                "Retry count",
                "Reconnection attempts before giving up.",
                SettingKind::Number {
                    min: 0.0,
                    max: 20.0,
                    step: 1.0,
                },
                SettingValue::U32(&mut self.retry_count),
            ),
            SettingEntry::new(
                "network.stream_buffer_mib",
                "Streaming buffer",
                "Backbuffer kept for streams in mebibytes.",
                SettingKind::Slider {
                    min: 0.0,
                    max: 256.0,
                    step: 4.0,
                    unit: "MiB",
                },
                SettingValue::U32(&mut self.stream_buffer_mib),
            ),
            SettingEntry::new(
                "network.protocols",
                "Preferred network protocols",
                "Restrict which protocols may be opened.",
                SettingKind::Dropdown {
                    options: PROTOCOL_OPTIONS.to_vec(),
                },
                SettingValue::Choice(&mut self.protocols),
            ),
            SettingEntry::new(
                "network.max_concurrent",
                "Max concurrent operations",
                "Maximum simultaneous network reads.",
                SettingKind::Number {
                    min: 1.0,
                    max: 32.0,
                    step: 1.0,
                },
                SettingValue::U32(&mut self.max_concurrent),
            ),
            SettingEntry::new(
                "network.user_agent",
                "User agent",
                "HTTP user agent string sent for network streams.",
                SettingKind::Text,
                SettingValue::Text(&mut self.user_agent),
            ),
            SettingEntry::new(
                "network.show_status",
                "Network playback status",
                "Show buffering and cache status in the info overlay.",
                SettingKind::Toggle,
                SettingValue::Bool(&mut self.show_status),
            ),
        ]
    }
}

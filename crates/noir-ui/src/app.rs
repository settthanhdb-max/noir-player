//! Application shell: navigation, top bar, global state, event plumbing.

use crate::dialogs::{DialogKind, DialogResult, FileDialogs};
use crate::i18n::Tr;
use crate::state::{PickRequest, QueueEntry, RepeatMode, Screen};
use crate::{player, playlist_panel, screens, settings_ui, theme};
use egui::{Color32, Context, CornerRadius, Rect, RichText, Ui};
use noir_core::events::PlaybackEvent;
use noir_core::state::PlaybackState;
use noir_library::LibraryService;
use noir_platform::PlatformPaths;
use noir_playback::engine::EngineCommand;
use noir_playback::{Capabilities, PlaybackController};
use noir_settings::schema::input::ActionBinding;
use noir_settings::schema::Settings;
use noir_settings::SettingsStore;
use std::time::{Duration, Instant};

/// Staged values for the playback settings overlay; committed when the user
/// presses Apply.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackTweaks {
    pub brightness: i32,
    pub contrast: i32,
    pub saturation: i32,
    pub gamma: i32,
    pub hue: i32,
    pub speed: f64,
    pub volume: u32,
    pub muted: bool,
}

impl Default for PlaybackTweaks {
    fn default() -> Self {
        Self {
            brightness: 0,
            contrast: 0,
            saturation: 0,
            gamma: 0,
            hue: 0,
            speed: 1.0,
            volume: 100,
            muted: false,
        }
    }
}

pub struct NoirApp {
    pub settings: Settings,
    pub store: SettingsStore,
    pub controller: PlaybackController,
    pub capabilities: Capabilities,
    pub engine_attached: bool,
    pub video_capable: bool,
    pub library: Option<LibraryService>,
    pub tr: Tr,
    pub base_ppp: f32,

    pub screen: Screen,
    pub queue: Vec<QueueEntry>,
    pub queue_index: Option<usize>,
    pub queue_search: String,
    pub queue_name_input: String,
    pub save_queue_prompt: bool,
    pub show_playlist: bool,
    pub show_info: bool,
    pub is_fullscreen: bool,
    pub scrub_position: Option<f64>,
    pub scrubbing: Option<f64>,
    pub viewport_rect: Option<Rect>,
    pub video_child_shown: bool,
    /// Expandable adjustments section inside the control bar (below the
    /// video, so playback is NOT paused while it is open).
    pub adjustments_expanded: bool,
    pub last_mouse_activity: Instant,
    /// Last OS cursor position seen over the player (physical client px).
    /// egui never receives pointer deltas over the native video child, so
    /// movement detection there is done by comparing polled positions.
    last_video_cursor: Option<(i32, i32)>,
    /// Was the last OS-cursor poll position inside the video viewport?
    pub(crate) last_poll_inside: bool,
    /// Number of mpv-observed mouse-over-video movement events received.
    pub(crate) mpv_activity_count: u64,
    /// Show live auto-hide diagnostics (toggle with F2).
    pub debug_info: bool,
    pub fill_mode: usize,
    pub tweaks: PlaybackTweaks,
    pub ipc_open_file: std::path::PathBuf,
    pub osd_message: Option<(String, Instant)>,
    pub error_message: Option<String>,
    pub dialogs: FileDialogs,
    pub pending_pick: Option<PickRequest>,

    pub settings_search: String,
    pub settings_category: usize,
    pub show_advanced: bool,
    pub capture_action: Option<ActionBinding>,
    pub library_search: String,
    pub library_sort: usize,
    pub library_filter: usize,
    pub scan_rx: Option<crossbeam_channel::Receiver<noir_library::LibraryScanUpdate>>,
    pub scan_status: Option<String>,
    pub confirm_clear_history: bool,
    pub confirm_exit: bool,
    pub ab_loop_state: Option<(f64, Option<f64>)>,

    last_theme_fingerprint: (theme::Palette, f32, f32, f32, bool, bool),
    was_focused: bool,
    auto_paused: bool,
    last_recorded_path: Option<String>,
}

impl NoirApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let paths = PlatformPaths::resolve();
        if let Err(e) = paths.ensure_dirs() {
            tracing::warn!("could not create data directories: {e}");
        }
        let store = SettingsStore::new(paths.settings_file());
        let settings = store.load();

        let engine = noir_playback::create_default_engine();
        let capabilities = engine.capabilities();
        let video_capable = capabilities.video;
        let controller = PlaybackController::spawn(engine);
        for cmd in noir_playback::settings_map::initial_commands(
            &settings.video,
            &settings.audio,
            &settings.subtitles,
            &settings.network,
            &settings.playback,
        ) {
            controller.execute(cmd);
        }

        let library = match LibraryService::open(&paths.database_file()) {
            Ok(lib) => Some(lib),
            Err(e) => {
                tracing::error!("library database unavailable: {e}");
                None
            }
        };

        let mut app = Self {
            tr: Tr::from_index(settings.general.language),
            settings,
            store,
            controller,
            capabilities,
            engine_attached: false,
            video_capable,
            library,
            screen: Screen::Home,
            queue: Vec::new(),
            queue_index: None,
            queue_search: String::new(),
            queue_name_input: String::new(),
            save_queue_prompt: false,
            show_playlist: false,
            show_info: false,
            is_fullscreen: false,
            scrub_position: None,
            scrubbing: None,
            viewport_rect: None,
            video_child_shown: false,
            adjustments_expanded: false,
            last_mouse_activity: Instant::now(),
            last_video_cursor: None,
            last_poll_inside: false,
            mpv_activity_count: 0,
            debug_info: false,
            fill_mode: 0,
            tweaks: PlaybackTweaks::default(),
            ipc_open_file: paths.data_dir.join("ipc_open.txt"),
            // Version splash: unambiguous proof of which build is running.
            osd_message: Some((
                format!("NOIR Player v{}", env!("CARGO_PKG_VERSION")),
                Instant::now(),
            )),
            error_message: None,
            dialogs: FileDialogs::default(),
            pending_pick: None,
            settings_search: String::new(),
            settings_category: 0,
            show_advanced: false,
            capture_action: None,
            library_search: String::new(),
            library_sort: 0,
            library_filter: 0,
            scan_rx: None,
            scan_status: None,
            confirm_clear_history: false,
            confirm_exit: false,
            ab_loop_state: None,
            last_theme_fingerprint: (theme::Palette::Dark, 1.0, 13.0, 4.0, false, false),
            base_ppp: 1.0,
            was_focused: true,
            auto_paused: false,
            last_recorded_path: None,
        };
        app.screen = Screen::from_startup_index(app.settings.appearance.startup_screen);
        app.base_ppp = cc.egui_ctx.pixels_per_point();
        theme::apply(
            &cc.egui_ctx,
            app.current_palette(),
            &app.settings.appearance,
            app.base_ppp,
        );
        app
    }

    pub fn current_palette(&self) -> theme::Palette {
        theme::Palette::from_theme_index(self.settings.appearance.theme, true)
    }

    // ---- playback helpers ----

    pub fn open_and_play(&mut self, path: String) {
        if self.settings.playlist.auto_add_opened && !self.queue.iter().any(|e| e.path == path) {
            self.queue.push(QueueEntry::from_path(path.clone()));
            self.queue_index = Some(self.queue.len() - 1);
        }
        let resume = if self.settings.playback.auto_resume {
            self.library
                .as_ref()
                .and_then(|lib| lib.resume_position(std::path::Path::new(&path)))
                .filter(|pos| *pos > self.settings.playback.resume_threshold_s as f64)
        } else {
            None
        };
        self.controller.open(path, resume);
        self.screen = Screen::Player;
        // Show the control bar immediately when media opens.
        self.mark_mouse_activity();
    }

    pub fn play_from_queue(&mut self, index: usize) {
        if let Some(entry) = self.queue.get(index) {
            self.queue_index = Some(index);
            let resume = if self.settings.playback.auto_resume {
                self.library
                    .as_ref()
                    .and_then(|lib| lib.resume_position(std::path::Path::new(&entry.path)))
                    .filter(|pos| *pos > self.settings.playback.resume_threshold_s as f64)
            } else {
                None
            };
            self.controller.open(entry.path.clone(), resume);
            self.screen = Screen::Player;
        }
    }

    pub fn load_playlist_into_queue(&mut self, paths: &[String]) {
        self.queue = paths.iter().map(QueueEntry::from_path).collect();
        self.queue_index = None;
    }

    pub fn play_next(&mut self) {
        self.advance(1);
    }

    pub fn play_previous(&mut self) {
        self.advance(-1);
    }

    fn advance(&mut self, direction: i32) {
        if self.queue.is_empty() {
            return;
        }
        let repeat = RepeatMode::from_index(self.settings.playlist.repeat_mode);
        if self.settings.playlist.shuffle {
            let next = (fastrand_index(self.queue.len())) as usize;
            self.play_from_queue(next);
            return;
        }
        let current = self.queue_index.unwrap_or(0) as i64;
        let mut next = current + direction as i64;
        let len = self.queue.len() as i64;
        if next < 0 {
            if repeat == RepeatMode::All {
                next = len - 1;
            } else {
                return;
            }
        } else if next >= len {
            match repeat {
                RepeatMode::All => next = 0,
                RepeatMode::One => next = current,
                RepeatMode::Off => {
                    self.controller.stop();
                    return;
                }
            }
        }
        self.play_from_queue(next as usize);
    }

    pub fn toggle_fullscreen(&mut self, ctx: &Context) {
        self.is_fullscreen = !self.is_fullscreen;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
        // Give instant visual feedback after toggling: the control bar shows
        // again immediately instead of staying hidden.
        self.mark_mouse_activity();
    }

    /// Entry point for files passed on the command line (file associations).
    pub fn new_with_files(cc: &eframe::CreationContext<'_>, files: &[String]) -> Self {
        let mut app = Self::new(cc);
        if let Some(first) = files.first() {
            app.load_playlist_into_queue(files);
            app.open_and_play(first.clone());
        }
        app
    }

    /// Any pointer movement over the player resets the auto-hide timer so the
    /// control bar reappears while the mouse is in use.
    pub fn mark_mouse_activity(&mut self) {
        self.last_mouse_activity = Instant::now();
    }

    /// Poll the OS cursor position and treat movement inside the video
    /// viewport as user activity. Required because WM_MOUSEMOVE over the
    /// embedded mpv child window goes to the child, so egui's own pointer
    /// deltas stay at zero there. Movement elsewhere on the player window is
    /// already detected through egui input, and movement in other programs
    /// must not resurrect the control bar.
    fn poll_video_cursor_activity(
        &mut self,
        parent: *mut std::ffi::c_void,
        rect: egui::Rect,
        ppp: f32,
    ) {
        let pos = noir_platform::windowing::cursor_client_pos(parent);
        let inside = pos
            .map(|(x, y)| {
                x >= (rect.min.x * ppp) as i32
                    && x <= (rect.max.x * ppp) as i32
                    && y >= (rect.min.y * ppp) as i32
                    && y <= (rect.max.y * ppp) as i32
            })
            .unwrap_or(false);
        self.last_poll_inside = inside;
        if inside && pos != self.last_video_cursor {
            self.mark_mouse_activity();
        }
        if pos.is_some() {
            self.last_video_cursor = pos;
        }
    }

    /// Whether the playback control bar should currently be drawn. It hides
    /// after `autohide_delay` seconds without mouse movement while a video
    /// plays, and always shows while idle/paused or when an overlay is open.
    pub fn controls_visible(&self, snapshot: &noir_playback::engine::EngineSnapshot) -> bool {
        if !self.settings.appearance.autohide_controls || self.adjustments_expanded {
            return true;
        }
        if self.screen != Screen::Player {
            return true;
        }
        if snapshot.state != PlaybackState::Playing {
            return true;
        }
        self.last_mouse_activity.elapsed()
            < Duration::from_secs_f32(self.settings.appearance.autohide_delay.max(0.5))
    }

    /// Expand the in-bar adjustments section, re-staging the current values.
    /// Playback keeps running: the section lives in the control bar below the
    /// video, outside the native video surface.
    pub fn begin_adjustments(&mut self, snapshot: &noir_playback::engine::EngineSnapshot) {
        let video = &self.settings.video;
        self.tweaks = PlaybackTweaks {
            brightness: video.brightness,
            contrast: video.contrast,
            saturation: video.saturation,
            gamma: video.gamma,
            hue: video.hue,
            speed: snapshot.speed,
            volume: snapshot.volume,
            muted: snapshot.muted,
        };
        self.adjustments_expanded = true;
        self.mark_mouse_activity();
    }

    pub fn end_adjustments(&mut self) {
        self.adjustments_expanded = false;
        self.mark_mouse_activity();
    }

    /// Commit the staged overlay values: video equalizer via the settings
    /// pipeline, speed/volume/mute as direct engine commands.
    pub fn apply_tweaks(&mut self) {
        let t = self.tweaks;
        {
            let video = &mut self.settings.video;
            video.brightness = t.brightness;
            video.contrast = t.contrast;
            video.saturation = t.saturation;
            video.gamma = t.gamma;
            video.hue = t.hue;
        }
        self.on_settings_changed();
        let playback = self.settings.playback.clone();
        self.controller
            .set_speed(playback.clamp_speed(t.speed as f32) as f64);
        self.controller.set_volume(playback.clamp_volume(t.volume));
        self.controller
            .execute(EngineCommand::SetMuted { muted: t.muted });
    }

    /// Persist the currently staged equalizer values (toolbar live tuning)
    /// without pausing playback or resending unrelated commands.
    pub fn persist_equalizer(&mut self) {
        let t = self.tweaks;
        {
            let video = &mut self.settings.video;
            video.brightness = t.brightness;
            video.contrast = t.contrast;
            video.saturation = t.saturation;
            video.gamma = t.gamma;
            video.hue = t.hue;
        }
        self.on_settings_changed();
    }
    pub fn reset_tweaks(&mut self) {
        let defaults = Settings::defaults();
        self.tweaks = PlaybackTweaks {
            brightness: defaults.video.brightness,
            contrast: defaults.video.contrast,
            saturation: defaults.video.saturation,
            gamma: defaults.video.gamma,
            hue: defaults.video.hue,
            speed: defaults.playback.default_speed as f64,
            volume: defaults.playback.default_volume,
            muted: false,
        };
    }

    /// Aspect/fill modes: 0 default, 1 fill (crop to window), then fixed
    /// aspect overrides.
    pub fn set_fill_mode(&mut self, mode: usize) {
        self.fill_mode = mode;
        let (aspect, panscan) = match mode {
            0 => (None, 0.0),
            1 => (None, 1.0),
            2 => (Some("16:9".to_string()), 0.0),
            3 => (Some("4:3".to_string()), 0.0),
            4 => (Some("1:1".to_string()), 0.0),
            _ => (Some("2.35:1".to_string()), 0.0),
        };
        self.controller.execute(EngineCommand::SetAspect { aspect });
        self.controller
            .execute(EngineCommand::SetPanScan { value: panscan });
    }

    /// Open media files handed off by a second instance (double-clicked from
    /// Explorer while the player is already running).
    pub fn process_open_requests(&mut self, ctx: &Context) {
        let Ok(content) = std::fs::read_to_string(&self.ipc_open_file) else {
            return;
        };
        let _ = std::fs::remove_file(&self.ipc_open_file);
        let files: Vec<String> = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        if files.is_empty() {
            return;
        }
        self.load_playlist_into_queue(&files);
        if let Some(first) = files.first() {
            self.open_and_play(first.clone());
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    pub fn rescan_all(&mut self) {
        let Some(lib) = self.library.clone() else {
            return;
        };
        for dir in lib.directories() {
            self.scan_rx = Some(lib.scan_directory(dir.into()));
        }
    }

    // ---- settings plumbing ----

    pub fn on_settings_changed(&mut self) {
        noir_settings::validation::normalize(&mut self.settings);
        if let Err(e) = self.store.save(&self.settings) {
            tracing::error!("failed to save settings: {e}");
            self.error_message = Some(format!("{}: {e}", self.tr.t("set.error")));
        }
        self.tr = Tr::from_index(self.settings.general.language);
        let a = &self.settings.appearance;
        self.last_theme_fingerprint = (
            self.current_palette(),
            a.ui_scale,
            a.font_size,
            a.corner_radius,
            a.compact_mode,
            a.dense_mode,
        );
        for cmd in noir_playback::settings_map::initial_commands(
            &self.settings.video,
            &self.settings.audio,
            &self.settings.subtitles,
            &self.settings.network,
            &self.settings.playback,
        ) {
            self.controller.execute(cmd);
        }
    }

    pub fn reset_category(&mut self, name: &str) {
        let defaults = Settings::defaults();
        match name {
            "general" => self.settings.general = defaults.general,
            "appearance" => self.settings.appearance = defaults.appearance,
            "playback" => self.settings.playback = defaults.playback,
            "video" => self.settings.video = defaults.video,
            "audio" => self.settings.audio = defaults.audio,
            "subtitles" => self.settings.subtitles = defaults.subtitles,
            "network" => self.settings.network = defaults.network,
            "playlist" => self.settings.playlist = defaults.playlist,
            "input" => self.settings.input = defaults.input,
            "advanced" => self.settings.advanced = defaults.advanced,
            _ => {}
        }
    }

    // ---- events ----

    fn drain_playback_events(&mut self) {
        for event in self.controller.drain_events() {
            match event {
                PlaybackEvent::EndOfFile => {
                    self.record_history_session();
                    self.on_playback_finished();
                }
                PlaybackEvent::Error(message) => {
                    tracing::error!("playback error: {message}");
                    self.error_message = Some(message);
                }
                PlaybackEvent::OsdMessage(text) => {
                    self.osd_message = Some((text, Instant::now()));
                }
                PlaybackEvent::Activity => {
                    // Diagnostic counter only: some mpv builds fire mouse-pos
                    // updates without real mouse movement, which kept the bar
                    // visible forever. Reveal now relies purely on real cursor
                    // displacement (poll_video_cursor_activity) and on real
                    // egui input (movement, clicks, keys).
                    self.mpv_activity_count += 1;
                }
                PlaybackEvent::FileLoaded { path } => {
                    if self.settings.playlist.auto_add_opened
                        && !self.queue.iter().any(|e| e.path == path)
                    {
                        self.queue.push(QueueEntry::from_path(path.clone()));
                    }
                    if let Some(index) = self.queue.iter().position(|e| e.path == path) {
                        self.queue_index = Some(index);
                    }
                }
                PlaybackEvent::MediaLoaded {
                    title, duration, ..
                } => {
                    if let Some(entry) = self
                        .queue
                        .iter_mut()
                        .find(|e| Some(&e.path) == self.controller.snapshot().current_path.as_ref())
                    {
                        entry.duration = Some(duration);
                        let _ = &title;
                    }
                }
                _ => {}
            }
        }
    }

    fn on_playback_finished(&mut self) {
        let repeat = RepeatMode::from_index(self.settings.playlist.repeat_mode);
        match self.settings.playback.completion_action {
            1 => self.advance(1), // play next
            2 => {
                // repeat file
                if let Some(path) = self.controller.snapshot().current_path {
                    self.controller.open(path, None);
                }
            }
            3 => {
                self.controller.stop();
                self.record_history_session();
                crate::app::request_exit();
            }
            _ => {
                if repeat == RepeatMode::One {
                    if let Some(path) = self.controller.snapshot().current_path {
                        self.controller.open(path, None);
                    }
                } else if !self.queue.is_empty() {
                    self.advance(1);
                }
            }
        }
    }

    fn record_history_session(&mut self) {
        let snapshot = self.controller.snapshot();
        let Some(path) = snapshot.current_path.clone() else {
            return;
        };
        if self.last_recorded_path.as_deref() == Some(path.as_str()) {
            return;
        }
        self.last_recorded_path = Some(path.clone());
        let policy = noir_library::history::HistoryPolicy {
            enabled: self.settings.playback.record_history,
            remember_position: self.settings.playback.remember_position_per_file,
            min_duration_s: 5.0,
        };
        let title = std::path::Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        if let Some(entry) = noir_library::history::build_entry(
            policy,
            &path,
            &title,
            snapshot.position,
            if snapshot.duration > 0.0 {
                Some(snapshot.duration)
            } else {
                None
            },
        ) {
            if let Some(lib) = &self.library {
                lib.record_history(&entry);
            }
        }
    }

    // ---- dialogs ----

    fn poll_dialogs(&mut self) {
        let Some((kind, context, result)) = self.dialogs.poll() else {
            return;
        };
        if matches!(result, DialogResult::Cancelled) {
            self.pending_pick = None;
            return;
        }
        match (kind, result) {
            (DialogKind::OpenMedia, DialogResult::Paths(paths)) => {
                let paths: Vec<String> = paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                if paths.is_empty() {
                    return;
                }
                for p in &paths {
                    if !self.queue.iter().any(|e| &e.path == p) {
                        self.queue.push(QueueEntry::from_path(p.clone()));
                    }
                }
                let first = paths[0].clone();
                self.open_and_play(first);
            }
            (DialogKind::PickDirectory, DialogResult::Directory(dir)) => {
                let dir_str = dir.to_string_lossy().to_string();
                match self.pending_pick.take() {
                    Some(PickRequest::Directory { key }) => {
                        self.set_text_setting(&key, dir_str);
                    }
                    Some(PickRequest::ScreenshotDir) => {
                        self.settings.general.screenshot_dir = dir_str;
                        self.on_settings_changed();
                    }
                    _ => match context.as_deref() {
                        Some("lib-add-dir") => {
                            if let Some(lib) = self.library.clone() {
                                self.scan_rx = Some(lib.scan_directory(dir));
                            }
                        }
                        Some("open-folder") => {
                            if let Some(lib) = self.library.clone() {
                                self.scan_rx = Some(lib.scan_directory(dir.clone()));
                            }
                            let entries: Vec<String> = noir_library::scanner::LibraryScanner::new()
                                .scan_blocking(&dir)
                                .iter()
                                .map(|item| item.path.to_string_lossy().to_string())
                                .collect();
                            if !entries.is_empty() {
                                self.load_playlist_into_queue(&entries);
                                self.play_from_queue(0);
                            }
                        }
                        _ => {}
                    },
                }
            }
            (DialogKind::OpenSubtitle, DialogResult::Paths(paths)) => {
                if let Some(sub) = paths.first() {
                    self.controller.execute(EngineCommand::LoadSubtitle {
                        path: sub.to_string_lossy().to_string(),
                    });
                }
            }
            (DialogKind::SaveM3u, DialogResult::Save(target)) => {
                let paths: Vec<String> = context
                    .and_then(|c| serde_json::from_str(&c).ok())
                    .unwrap_or_else(|| self.queue.iter().map(|e| e.path.clone()).collect());
                let entries: Vec<noir_library::m3u::M3uEntry> = paths
                    .iter()
                    .map(|p| noir_library::m3u::M3uEntry {
                        path: p.clone(),
                        title: std::path::Path::new(p)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string()),
                    })
                    .collect();
                let content = noir_library::m3u::serialize(&entries, None);
                if let Err(e) = std::fs::write(&target, content) {
                    tracing::error!("failed to write M3U: {e}");
                    self.error_message = Some(e.to_string());
                }
            }
            (DialogKind::OpenM3u, DialogResult::Paths(paths)) => {
                if let Some(source) = paths.first() {
                    match std::fs::read_to_string(source) {
                        Ok(content) => {
                            let base = source.parent().map(|p| p.to_path_buf());
                            let items: Vec<String> =
                                noir_library::m3u::parse(&content, base.as_deref())
                                    .into_iter()
                                    .map(|e| e.path)
                                    .collect();
                            self.load_playlist_into_queue(&items);
                            if !items.is_empty() {
                                self.play_from_queue(0);
                            }
                        }
                        Err(e) => {
                            tracing::error!("failed to read M3U: {e}");
                            self.error_message = Some(e.to_string());
                        }
                    }
                }
            }
            (DialogKind::SaveJson, DialogResult::Save(target)) => match context.as_deref() {
                Some("config-export") => {
                    if let Err(e) = self.store.export_to(&self.settings, &target) {
                        tracing::error!("config export failed: {e}");
                        self.error_message = Some(e.to_string());
                    }
                }
                Some("keys-export") => {
                    let json = serde_json::to_string_pretty(&self.settings.input.bindings)
                        .unwrap_or_else(|_| "[]".into());
                    if let Err(e) = std::fs::write(&target, json) {
                        self.error_message = Some(e.to_string());
                    }
                }
                _ => {}
            },
            (DialogKind::OpenJson, DialogResult::Paths(paths)) => {
                let Some(source) = paths.first() else { return };
                match context.as_deref() {
                    Some("config-import") => match self.store.import_from(source) {
                        Ok(imported) => {
                            self.settings = imported;
                            self.on_settings_changed();
                        }
                        Err(e) => {
                            tracing::error!("config import failed: {e}");
                            self.error_message = Some(e.to_string());
                        }
                    },
                    Some("keys-import") => match std::fs::read_to_string(source)
                        .map_err(|e| e.to_string())
                        .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
                    {
                        Ok(bindings) => {
                            self.settings.input.bindings = bindings;
                            self.on_settings_changed();
                        }
                        Err(e) => self.error_message = Some(e.to_string()),
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn set_text_setting(&mut self, key: &str, value: String) {
        match key {
            "general.default_media_dir" => self.settings.general.default_media_dir = value,
            "general.screenshot_dir" => self.settings.general.screenshot_dir = value,
            _ => {
                tracing::warn!("unhandled directory pick for key {key}");
                return;
            }
        }
        self.on_settings_changed();
    }

    // ---- input ----

    fn handle_input(&mut self, ctx: &Context) {
        let events: Vec<egui::Event> = ctx.input(|i| i.events.clone());

        // F2 toggles the auto-hide diagnostics readout in the top bar.
        if events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: egui::Key::F2,
                    pressed: true,
                    ..
                }
            )
        }) {
            self.debug_info = !self.debug_info;
        }

        // Any key press or mouse click also resets the auto-hide timer: the
        // control bar must come back on ANY deliberate input, not only mouse
        // movement (e.g. when the OS cursor never leaves the video surface).
        let input_activity = ctx.input(|i| {
            i.pointer.any_pressed()
                || i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
        });
        if input_activity {
            self.mark_mouse_activity();
        }

        // Shortcut capture mode: the next key press rebinds the action.
        if let Some(action) = self.capture_action {
            for event in &events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if let Some(combo) = crate::shortcuts::combo_from(*modifiers, *key) {
                        let mut bindings = self.settings.input.bindings.clone();
                        let mut registry = noir_settings::schema::input::ShortcutRegistry::new(
                            std::mem::take(&mut bindings),
                        );
                        registry.assign(action, combo);
                        self.settings.input.bindings = registry.bindings().to_vec();
                        self.capture_action = None;
                        self.on_settings_changed();
                    }
                }
            }
            return;
        }

        let registry = noir_settings::schema::input::ShortcutRegistry::new(
            self.settings.input.bindings.clone(),
        );
        for (modifiers, key) in crate::shortcuts::pressed_combos(&events) {
            if let Some(action) = registry.lookup(modifiers, &key) {
                self.handle_action(ctx, action);
            }
        }

        // Mouse wheel over the viewport.
        let wheel = ctx.input(|i| i.smooth_scroll_delta.y);
        if wheel != 0.0 && self.viewport_hovered(ctx) {
            let snapshot = self.controller.snapshot();
            match self.settings.input.wheel_action {
                1 => {
                    let step = self.settings.input.wheel_seek_s as f64;
                    self.controller
                        .seek_relative(if wheel > 0.0 { step } else { -step });
                }
                2 => {
                    let factor = if wheel > 0.0 { 1.1 } else { 1.0 / 1.1 };
                    self.controller.set_speed(
                        self.settings
                            .playback
                            .clamp_speed(snapshot.speed as f32 * factor)
                            as f64,
                    );
                }
                _ => {
                    let delta = if wheel > 0.0 { 5 } else { -5 };
                    self.controller.set_volume(
                        self.settings
                            .playback
                            .clamp_volume((snapshot.volume as i32 + delta).max(0) as u32),
                    );
                }
            }
        }

        // Drag-and-drop.
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let paths: Vec<String> = dropped
                .iter()
                .map(|f| f.path().to_string_lossy().to_string())
                .collect();
            if !paths.is_empty() {
                for p in &paths {
                    if !self.queue.iter().any(|e| &e.path == p) {
                        self.queue.push(QueueEntry::from_path(p.clone()));
                    }
                }
                let first = paths[0].clone();
                self.open_and_play(first);
            }
        }

        // Files handed off by a second instance.
        self.process_open_requests(ctx);

        // Confirm-exit intercept.
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.settings.general.confirm_exit && !self.confirm_exit {
            self.confirm_exit = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
    }

    fn viewport_hovered(&self, ctx: &Context) -> bool {
        self.viewport_rect
            .map(|rect| {
                ctx.input(|i| {
                    i.pointer
                        .latest_pos()
                        .map(|p| rect.contains(p))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    pub fn handle_action(&mut self, ctx: &Context, action: ActionBinding) {
        let playback = self.settings.playback.clone();
        match action {
            ActionBinding::PlayPause => self.controller.toggle_play(),
            ActionBinding::SeekBackward => self
                .controller
                .seek_relative(-playback.seek_backward_s as f64),
            ActionBinding::SeekForward => self
                .controller
                .seek_relative(playback.seek_forward_s as f64),
            ActionBinding::LongSeekBackward => {
                self.controller.seek_relative(-playback.long_seek_s as f64)
            }
            ActionBinding::LongSeekForward => {
                self.controller.seek_relative(playback.long_seek_s as f64)
            }
            ActionBinding::VolumeUp => {
                let v = self.controller.snapshot().volume;
                self.controller.set_volume(playback.clamp_volume(v + 5));
            }
            ActionBinding::VolumeDown => {
                let v = self.controller.snapshot().volume;
                self.controller.set_volume(v.saturating_sub(5));
            }
            ActionBinding::Mute => self.controller.toggle_mute(),
            ActionBinding::Fullscreen => self.toggle_fullscreen(ctx),
            ActionBinding::OpenFile => {
                self.dialogs
                    .request(DialogKind::OpenMedia, Some("open-media".into()));
            }
            ActionBinding::OpenFolder => {
                self.dialogs
                    .request(DialogKind::PickDirectory, Some("open-folder".into()));
            }
            ActionBinding::PlaylistPanel => self.show_playlist = !self.show_playlist,
            ActionBinding::SubtitleMenu => {
                self.dialogs.request(DialogKind::OpenSubtitle, None);
            }
            ActionBinding::AudioTrackMenu => {
                let tracks = self.controller.snapshot().tracks;
                let audio: Vec<_> = tracks
                    .iter()
                    .filter(|t| t.kind == noir_core::models::TrackKind::Audio)
                    .cloned()
                    .collect();
                if let Some(next) = next_track(&audio) {
                    self.controller.execute(EngineCommand::SelectTrack {
                        kind: noir_core::models::TrackKind::Audio,
                        id: next.id,
                    });
                }
            }
            ActionBinding::SpeedDown => {
                let s = self.controller.snapshot().speed;
                self.controller.set_speed(
                    playback.clamp_speed((s - playback.speed_increment as f64) as f32) as f64,
                );
            }
            ActionBinding::SpeedUp => {
                let s = self.controller.snapshot().speed;
                self.controller.set_speed(
                    playback.clamp_speed((s + playback.speed_increment as f64) as f32) as f64,
                );
            }
            ActionBinding::SpeedReset => {
                self.controller.set_speed(playback.default_speed as f64);
            }
            ActionBinding::FrameStepBackward => {
                self.controller
                    .execute(EngineCommand::FrameStep { direction: -1 });
            }
            ActionBinding::FrameStepForward => {
                self.controller
                    .execute(EngineCommand::FrameStep { direction: 1 });
            }
            ActionBinding::PreviousMedia => self.play_previous(),
            ActionBinding::NextMedia => self.play_next(),
            ActionBinding::ChapterPrevious => {
                let chapters = self.controller.snapshot().chapters;
                let pos = self.controller.snapshot().position;
                if let Some(ch) = chapters.iter().rev().find(|c| c.start < pos - 0.5) {
                    self.controller.seek_absolute(ch.start);
                }
            }
            ActionBinding::ChapterNext => {
                let chapters = self.controller.snapshot().chapters;
                let pos = self.controller.snapshot().position;
                if let Some(ch) = chapters.iter().find(|c| c.start > pos + 0.5) {
                    self.controller.seek_absolute(ch.start);
                }
            }
            ActionBinding::Screenshot => {
                let dir = if self.settings.general.screenshot_dir.is_empty() {
                    None
                } else {
                    Some(self.settings.general.screenshot_dir.clone())
                };
                self.controller
                    .execute(EngineCommand::Screenshot { path: dir });
            }
            ActionBinding::InfoOverlay => self.show_info = !self.show_info,
            ActionBinding::AbRepeatMark => {
                let snapshot = self.controller.snapshot();
                match self.ab_loop_state {
                    None => {
                        self.ab_loop_state = Some((snapshot.position, None));
                        self.osd_message =
                            Some((format!("A @ {:.1}s", snapshot.position), Instant::now()));
                    }
                    Some((a, None)) => {
                        self.ab_loop_state = Some((a, Some(snapshot.position)));
                        self.controller.execute(EngineCommand::SetAbLoop {
                            a: Some(a),
                            b: Some(snapshot.position),
                        });
                        self.osd_message = Some((
                            format!("A-B {:.1}–{:.1}s", a, snapshot.position),
                            Instant::now(),
                        ));
                    }
                    Some(_) => {
                        self.ab_loop_state = None;
                        self.controller
                            .execute(EngineCommand::SetAbLoop { a: None, b: None });
                        self.osd_message = Some(("A-B off".into(), Instant::now()));
                    }
                }
            }
            ActionBinding::Stop => self.controller.stop(),
            ActionBinding::ExitFullscreenCloseDialog => {
                if self.is_fullscreen {
                    self.toggle_fullscreen(ctx);
                } else if self.capture_action.is_some() {
                    self.capture_action = None;
                }
            }
        }
    }

    // ---- background bookkeeping ----

    fn background_tick(&mut self) {
        // Scan progress.
        if let Some(rx) = &self.scan_rx {
            let mut status = None;
            let mut done = false;
            while let Ok(update) = rx.try_recv() {
                match update {
                    noir_library::LibraryScanUpdate::Found(n) => {
                        let mut s = self.scan_status.take().unwrap_or_default();
                        s.push_str(&format!("+{n} "));
                        status = Some(s);
                    }
                    noir_library::LibraryScanUpdate::Finished { found, .. } => {
                        status = Some(format!("{}: +{found}", self.tr.t("lib.scan_status")));
                        done = true;
                    }
                    noir_library::LibraryScanUpdate::Failed { error, .. } => {
                        status = Some(error);
                        done = true;
                    }
                    noir_library::LibraryScanUpdate::Persisted => {}
                }
            }
            if let Some(s) = status {
                self.scan_status = Some(s);
            }
            if done {
                self.scan_rx = None;
            }
        }

        // Focus-loss pause.
        let focused = egui_focus();
        if self.was_focused && !focused {
            if self.settings.playback.pause_on_focus_loss
                && self.controller.state() == PlaybackState::Playing
            {
                self.controller.execute(EngineCommand::Pause);
                self.auto_paused = true;
            }
        } else if !self.was_focused && focused && self.auto_paused {
            if self.settings.playback.resume_on_focus {
                self.controller.toggle_play();
            }
            self.auto_paused = false;
        }
        self.was_focused = focused;

        // Expire OSD messages.
        if let Some((_, at)) = self.osd_message {
            if at.elapsed()
                > Duration::from_secs_f32(self.settings.appearance.osd_duration.max(0.5))
            {
                self.osd_message = None;
            }
        }
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        if self.is_fullscreen {
            return;
        }
        egui::Panel::top("top_bar")
            .frame(
                egui::Frame::default()
                    .fill(crate::theme::Palette::Dark.colors().surface)
                    .inner_margin(egui::Margin::symmetric(10, 5)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.settings.appearance.show_logo {
                        ui.label(RichText::new("NOIR").strong().monospace());
                    }
                    let snapshot = self.controller.snapshot();
                    let title = snapshot
                        .current_path
                        .as_ref()
                        .map(|p| {
                            std::path::Path::new(p)
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| p.clone())
                        })
                        .unwrap_or_else(|| self.tr.t("app.name").to_string());
                    ui.label(
                        RichText::new(&title)
                            .color(crate::theme::Palette::Dark.colors().text_secondary),
                    )
                    .on_hover_text(title);
                    ui.label(
                        RichText::new(player::state_label(self, &snapshot.state))
                            .small()
                            .color(crate::theme::Palette::Dark.colors().text_muted),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Live auto-hide diagnostics (F2): proves whether
                        // mouse activity over the video is being detected.
                        if self.debug_info {
                            let since = self.last_mouse_activity.elapsed().as_secs_f32();
                            ui.label(
                                RichText::new(format!(
                                    "act {:.1}s | poll_in {} | mpv_ev {} | cd {} ah {} d {:.1} | fs {}",
                                    since,
                                    self.last_poll_inside,
                                    self.mpv_activity_count,
                                    self.settings.appearance.show_controls,
                                    self.settings.appearance.autohide_controls,
                                    self.settings.appearance.autohide_delay,
                                    self.is_fullscreen
                                ))
                                .small()
                                .monospace()
                                .color(crate::theme::Palette::Dark.colors().text_secondary),
                            );
                        }
                        // Persistent build stamp: always visible in every
                        // screen (also during video playback, since the top
                        // bar is outside the native video area). A running
                        // exe without it is a stale build.
                        ui.label(
                            RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                                .small()
                                .monospace()
                                .color(crate::theme::Palette::Dark.colors().text_muted),
                        );
                        for screen in Screen::ALL {
                            let label = self.tr.t(screen.key());
                            if ui.selectable_label(self.screen == screen, label).clicked() {
                                self.screen = screen;
                            }
                        }
                        if ui
                            .selectable_label(self.show_playlist, self.tr.t("ctrl.playlist"))
                            .clicked()
                        {
                            self.show_playlist = !self.show_playlist;
                        }
                    });
                });
            });
    }

    fn confirm_exit_dialog(&mut self, ctx: &Context) {
        if !self.confirm_exit {
            return;
        }
        egui::Window::new(self.tr.t("misc.confirm_exit"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(self.tr.t("misc.exit")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button(self.tr.t("misc.cancel")).clicked() {
                        self.confirm_exit = false;
                    }
                });
            });
    }

    fn error_toast(&mut self, ctx: &Context) {
        let Some(message) = self.error_message.clone() else {
            return;
        };
        egui::Area::new(egui::Id::unique("error_toast"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 36.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(crate::theme::Palette::Dark.colors().elevated)
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0x52, 0x52, 0x5B)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&message).small());
                            if crate::icons::icon_button_small(
                                ui,
                                crate::icons::Icon::Close,
                                "Dismiss",
                            ) {
                                self.error_message = None;
                            }
                        });
                    });
            });
    }

    fn osd(&mut self, ctx: &Context) {
        let Some((message, _)) = self.osd_message.clone() else {
            return;
        };
        if !self.settings.appearance.osd_visible {
            return;
        }
        egui::Area::new(egui::Id::unique("osd"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -70.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(Color32::from_rgba_unmultiplied(0x09, 0x09, 0x0B, 200))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.label(RichText::new(message).small());
                    });
            });
    }
}

thread_local! {
    static EXIT_REQUEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn request_exit() {
    EXIT_REQUEST.with(|c| c.set(true));
}

fn exit_requested() -> bool {
    EXIT_REQUEST.with(|c| c.get())
}

fn fastrand_index(len: usize) -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    if len == 0 {
        0
    } else {
        nanos % len as u32
    }
}

fn next_track(tracks: &[noir_core::models::TrackInfo]) -> Option<noir_core::models::TrackInfo> {
    let selected = tracks.iter().position(|t| t.selected)?;
    let next = (selected + 1) % tracks.len().max(1);
    tracks.get(next).cloned()
}

fn egui_focus() -> bool {
    // Focus state is provided by the frame loop; stored via LAST_FOCUS.
    LAST_FOCUS.with(|c| c.get())
}

thread_local! {
    static LAST_FOCUS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

impl eframe::App for NoirApp {
    fn logic(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        LAST_FOCUS.with(|c| c.set(focused));

        self.drain_playback_events();
        self.poll_dialogs();
        self.background_tick();

        // Keep painting while media is playing so position updates smoothly.
        let state = self.controller.state();
        if matches!(
            state,
            PlaybackState::Playing | PlaybackState::Buffering | PlaybackState::Loading
        ) {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // One-time native window attachment for the mpv backend.
        if !self.engine_attached && self.video_capable {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = frame.window_handle() {
                let ptr = match handle.as_raw() {
                    RawWindowHandle::Win32(w) => w.hwnd.get() as *mut std::ffi::c_void,
                    RawWindowHandle::Xlib(x) => x.window as *mut std::ffi::c_void,
                    RawWindowHandle::AppKit(a) => a.ns_view.as_ptr(),
                    _ => std::ptr::null_mut(),
                };
                if !ptr.is_null() {
                    match self.controller.engine().attach_window(ptr) {
                        Ok(()) => {
                            self.engine_attached = true;
                            tracing::info!("playback engine attached to native window");
                        }
                        Err(e) => {
                            tracing::error!("failed to attach playback engine to window: {e}");
                        }
                    }
                }
            }
        }

        if exit_requested() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Live theme application when appearance settings change.
        let a = &self.settings.appearance;
        let fingerprint = (
            self.current_palette(),
            a.ui_scale,
            a.font_size,
            a.corner_radius,
            a.compact_mode,
            a.dense_mode,
        );
        if fingerprint != self.last_theme_fingerprint {
            self.last_theme_fingerprint = fingerprint;
            theme::apply(&ctx, fingerprint.0, a, self.base_ppp);
        }

        self.handle_input(&ctx);
        self.top_bar(ui);

        // Playlist side panel.
        if self.show_playlist && !self.is_fullscreen {
            egui::Panel::right("playlist_panel")
                .resizable(true)
                .default_size(self.settings.appearance.playlist_width.max(200.0))
                .show(ui, |ui| {
                    playlist_panel::panel(self, ui);
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(crate::theme::Palette::Dark.colors().background))
            .show(ui, |ui| match self.screen {
                Screen::Home => screens::home::screen(self, ui),
                Screen::Player => player::screen(self, ui),
                Screen::Library => screens::library::screen(self, ui),
                Screen::Playlists => screens::playlists::screen(self, ui),
                Screen::History => screens::history::screen(self, ui),
                Screen::Settings => settings_ui::screen(self, ui),
            });

        self.osd(&ctx);
        self.error_toast(&ctx);
        self.confirm_exit_dialog(&ctx);

        // Keep the embedded video surface aligned with the viewport rect and
        // visible only while the player screen is showing media. On any other
        // screen the child window is hidden so it can never overlap or cover
        // unrelated UI.
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let snapshot = self.controller.snapshot();
        let has_media = snapshot
            .current_path
            .as_deref()
            .map(|p| !p.trim().is_empty())
            .unwrap_or(false);
        let want_video = self.engine_attached
            && self.screen == Screen::Player
            && has_media
            && !matches!(snapshot.state, PlaybackState::Error(_));
        if !want_video {
            self.viewport_rect = None;
            self.last_video_cursor = None;
        }
        if self.engine_attached {
            if let Ok(handle) = frame.window_handle() {
                let parent = match handle.as_raw() {
                    RawWindowHandle::Win32(w) => w.hwnd.get() as *mut std::ffi::c_void,
                    _ => std::ptr::null_mut(),
                };
                if !parent.is_null() {
                    if want_video {
                        if let Some(rect) = self.viewport_rect {
                            if !self.video_child_shown
                                && noir_platform::windowing::set_video_child_visible(parent, true)
                            {
                                self.video_child_shown = true;
                            }
                            let ppp = ctx.pixels_per_point();
                            // Movement over the video area never reaches egui
                            // (the native child window swallows it), so poll
                            // the OS cursor to keep the auto-hide honest.
                            self.poll_video_cursor_activity(parent, rect, ppp);
                            let _ = noir_platform::windowing::position_video_child(
                                parent,
                                (rect.min.x * ppp) as i32,
                                (rect.min.y * ppp) as i32,
                                (rect.width() * ppp) as i32,
                                (rect.height() * ppp) as i32,
                            );
                        }
                    } else if self.video_child_shown
                        && noir_platform::windowing::set_video_child_visible(parent, false)
                    {
                        self.video_child_shown = false;
                    }
                }
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.record_history_session();
        if let Err(e) = self.store.save(&self.settings) {
            tracing::error!("failed to save settings on exit: {e}");
        }
        self.controller.shutdown();
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = crate::theme::Palette::Dark.colors().background;
        [
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            1.0,
        ]
    }
}

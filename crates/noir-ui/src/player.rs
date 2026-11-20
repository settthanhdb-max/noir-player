//! Player screen: auto-hiding control bar below the video (timeline,
//! transport, volume, expandable "more settings" section). Right-click does
//! nothing by design, and adjustments never pause playback: everything lives
//! in the bar below the video, outside the native video surface.

use crate::app::NoirApp;
use crate::theme::Palette;
use egui::{Align2, Color32, CornerRadius, Rect, RichText, Sense, Ui, Vec2};
use noir_core::models::{TrackInfo, TrackKind};
use noir_core::state::PlaybackState;

type EngineSnapshot = noir_playback::engine::EngineSnapshot;

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    let appearance = app.settings.appearance.clone();
    let snapshot = app.controller.snapshot();
    let controls_visible = app.controls_visible(&snapshot);

    // Any pointer movement on the player screen resets the auto-hide timer, so
    // the control bar reappears as soon as the mouse moves and hides again
    // after the configured delay (default 10 s) of inactivity. (Movement over
    // the video itself is detected separately via OS cursor polling; the
    // native video child swallows egui's pointer events there.)
    if ui.input(|i| i.pointer.delta().length_sq() > 0.0) {
        app.mark_mouse_activity();
    }

    let avail = ui.available_size();
    let show_info = app.show_info && appearance.show_info_overlay;
    let info_text = if show_info {
        info_lines(app, &snapshot).join("\n")
    } else {
        String::new()
    };
    let info_strip_height = if show_info {
        (info_text.lines().count() as f32 * 15.0 + 14.0).min(220.0)
    } else {
        0.0
    };
    // Control strip height: transport bar (or transport + expanded section).
    // Always reserved while controls are enabled so the video rect does not
    // jump when the auto-hiding bar appears or disappears. Values are in egui
    // points; egui converts to physical pixels via pixels_per_point when the
    // mpv child window is positioned, so no ppp scaling is needed here.
    let control_strip_height = if appearance.show_controls {
        if app.adjustments_expanded {
            // 64 collapsed bar + room for the expanded adjustments section.
            374.0
        } else {
            64.0
        }
    } else {
        0.0
    };
    let info_at_bottom = show_info && appearance.osd_position >= 2;
    let viewport_height = (avail.y - info_strip_height - control_strip_height).max(120.0);

    // The embedded mpv surface only ever covers the viewport rect: the info
    // and control panels live in dedicated strips outside it and can never be
    // covered by the native video window.
    if show_info && !info_at_bottom {
        info_strip(ui, avail.x, info_strip_height, &info_text);
    }

    let (viewport_rect, _) =
        ui.allocate_exact_size(Vec2::new(avail.x, viewport_height), Sense::click_and_drag());

    // Viewport background: pure black so letterboxing is seamless.
    ui.painter().rect_filled(viewport_rect, 0, Color32::BLACK);

    let has_media = snapshot
        .current_path
        .as_deref()
        .map(|p| !p.trim().is_empty())
        .unwrap_or(false);

    if let PlaybackState::Error(message) = &snapshot.state {
        egui::Area::new(egui::Id::unique("viewport_error"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(app.tr.t("state.error")).size(16.0).strong());
                    ui.label(
                        RichText::new(message)
                            .size(12.0)
                            .color(Color32::from_rgb(0xA1, 0xA1, 0xAA)),
                    );
                });
            });
    } else if !has_media {
        egui::Area::new(egui::Id::unique("viewport_hint"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(app.tr.t("misc.no_video")).size(16.0));
                    ui.label(
                        RichText::new(app.tr.t("misc.drop_here"))
                            .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
                    );
                });
            });
    } else if !app.engine_attached && app.video_capable {
        egui::Area::new(egui::Id::unique("viewport_loading"))
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new(app.tr.t("state.loading")).size(14.0));
            });
    }

    app.viewport_rect = Some(viewport_rect);

    // Viewport interactions: movement over the video keeps the control bar
    // visible, left click toggles pause, double click toggles fullscreen.
    // Right click is intentionally inert so stray clicks never interrupt
    // viewing.
    let interaction = ui.interact(
        viewport_rect,
        egui::Id::unique("video_viewport"),
        Sense::click_and_drag(),
    );
    if interaction.hovered() {
        app.mark_mouse_activity();
    }
    if interaction.clicked() {
        app.mark_mouse_activity();
        app.controller.toggle_play();
    }
    if interaction.double_clicked() {
        app.mark_mouse_activity();
        app.toggle_fullscreen(ui.ctx());
    }

    // Information strip (top or bottom depending on the OSD position setting).
    if info_at_bottom {
        info_strip(ui, avail.x, info_strip_height, &info_text);
    }

    // Control bar below the viewport, outside the native video rect. It stays
    // hidden while the mouse rests during playback and reappears on movement.
    //
    // IMPORTANT: the code below used to allocate the strip first and then
    // paint the bar afterwards; allocate_exact_size ADVANCES the layout cursor
    // past the reserved strip, so the bar was drawn below the window's bottom
    // edge and clipped off-screen entirely (this is why the "hotbar" never
    // showed, even though the strip itself rendered). allocate_ui draws INSIDE
    // the reserved region, so the bar can never fall outside the window.
    if appearance.show_controls {
        let strip = ui.allocate_ui(Vec2::new(avail.x, control_strip_height), |ui| {
            if controls_visible {
                let colors = Palette::Dark.colors();
                egui::Frame::default()
                    .fill(colors.surface)
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .stroke(egui::Stroke::new(1.0, colors.border))
                    .show(ui, |ui| control_panel(app, ui, &snapshot));
            } else {
                // The bar is hidden (auto-hide timer expired during playback):
                // keep the strip alive with a pinned summon button, a persistent
                // version stamp and (F2) the live auto-hide diagnostics. All of
                // this is visible even in fullscreen, when the top bar is gone.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::icons::icon_button_small(
                        ui,
                        crate::icons::Icon::Up,
                        app.tr.t("ctrl.show_controls"),
                    ) {
                        app.mark_mouse_activity();
                    }
                    ui.label(
                        RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                            .small()
                            .monospace()
                            .color(Palette::Dark.colors().text_muted),
                    );
                    if app.debug_info {
                        let since = app.last_mouse_activity.elapsed().as_secs_f32();
                        ui.label(
                            RichText::new(format!(
                                "act {:.1}s | poll_in {} | mpv_ev {} | ah {} d {:.1} | fs {}",
                                since,
                                app.last_poll_inside,
                                app.mpv_activity_count,
                                app.settings.appearance.autohide_controls,
                                app.settings.appearance.autohide_delay,
                                app.is_fullscreen
                            ))
                            .small()
                            .monospace()
                            .color(Palette::Dark.colors().text_secondary),
                        );
                    }
                });
            }
        });
        if strip.response.hovered() {
            app.mark_mouse_activity();
        }
    }
}

/// The control bar below the video: time + seek bar + duration on the first
/// row, transport/volume/more-settings on the second.
fn control_panel(app: &mut NoirApp, ui: &mut Ui, snapshot: &EngineSnapshot) {
    let duration = snapshot.duration.max(0.0);
    let position = app.scrub_position.unwrap_or(snapshot.position);

    // Row 1: position — seek bar — duration.
    ui.horizontal(|ui| {
        ui.monospace(format_time(position));
        let mut scrub = app.scrub_position;
        timeline(ui, position, duration, &mut scrub, &mut app.scrubbing);
        app.scrub_position = scrub;
        if let Some(target) = app.scrubbing {
            if !ui.input(|i| i.pointer.primary_down()) {
                app.controller
                    .execute(noir_playback::engine::EngineCommand::SeekAbsolute {
                        position: target,
                    });
                app.scrubbing = None;
                app.scrub_position = None;
                app.mark_mouse_activity();
            }
        }
        ui.monospace(format_time(duration));
    });

    // Row 2: transport, volume, expand toggle, screen toggles.
    let playback = app.settings.playback.clone();
    let max_volume = match app.settings.playback.max_volume_policy {
        1 => 130u32,
        2 => 200,
        _ => 100,
    };
    ui.horizontal(|ui| {
        let tr = app.tr;
        if crate::icons::icon_button(ui, crate::icons::Icon::Prev, tr.t("ctrl.prev")) {
            app.play_previous();
        }
        let (icon, tip) = match snapshot.state {
            PlaybackState::Playing | PlaybackState::Buffering => {
                (crate::icons::Icon::Pause, tr.t("ctrl.pause"))
            }
            _ => (crate::icons::Icon::Play, tr.t("ctrl.play")),
        };
        if crate::icons::icon_button(ui, icon, tip) {
            app.controller.toggle_play();
        }
        if crate::icons::icon_button(ui, crate::icons::Icon::Stop, tr.t("ctrl.stop")) {
            app.controller.stop();
        }
        if crate::icons::icon_button(ui, crate::icons::Icon::Next, tr.t("ctrl.next")) {
            app.play_next();
        }
        if crate::icons::icon_button(ui, crate::icons::Icon::StepBack, tr.t("ctrl.step_back")) {
            app.controller
                .execute(noir_playback::engine::EngineCommand::FrameStep { direction: -1 });
        }
        if crate::icons::icon_button(ui, crate::icons::Icon::StepFwd, tr.t("ctrl.step_fwd")) {
            app.controller
                .execute(noir_playback::engine::EngineCommand::FrameStep { direction: 1 });
        }

        ui.separator();

        let (mute_icon, mute_tip) = if snapshot.muted {
            (crate::icons::Icon::Muted, tr.t("ctrl.unmute"))
        } else {
            (crate::icons::Icon::Volume, tr.t("ctrl.mute"))
        };
        if crate::icons::icon_button(ui, mute_icon, mute_tip) {
            app.controller.toggle_mute();
        }
        let mut volume = snapshot.volume as f32;
        let vol = ui
            .allocate_ui(egui::vec2(90.0, 20.0), |ui| {
                ui.add(egui::Slider::new(&mut volume, 0.0..=max_volume as f32).show_value(false))
            })
            .inner;
        if vol.changed() {
            app.controller
                .set_volume(playback.clamp_volume(volume as u32));
        }

        ui.separator();

        // Expandable "more settings": visuals, speed, scaling, audio. Playback
        // is never paused for it — everything is below the video surface.
        let chevron = if app.adjustments_expanded {
            crate::icons::Icon::Down
        } else {
            crate::icons::Icon::Up
        };
        if crate::icons::icon_button(ui, chevron, tr.t("ctrl.more")) {
            if app.adjustments_expanded {
                app.end_adjustments();
            } else {
                app.begin_adjustments(snapshot);
            }
        }
        if ui.small_button(tr.t("menu.playback_settings")).clicked() {
            if app.adjustments_expanded {
                app.end_adjustments();
            } else {
                app.begin_adjustments(snapshot);
            }
        }

        ui.separator();

        if crate::icons::icon_button(ui, crate::icons::Icon::Playlist, tr.t("ctrl.playlist")) {
            app.show_playlist = !app.show_playlist;
        }
        if crate::icons::icon_button(ui, crate::icons::Icon::Info, tr.t("ctrl.info")) {
            app.show_info = !app.show_info;
        }
        let (fs_icon, fs_tip) = if app.is_fullscreen {
            (
                crate::icons::Icon::ExitFullscreen,
                tr.t("ctrl.exit_fullscreen"),
            )
        } else {
            (crate::icons::Icon::Fullscreen, tr.t("ctrl.fullscreen"))
        };
        if crate::icons::icon_button(ui, fs_icon, fs_tip) {
            app.toggle_fullscreen(ui.ctx());
        }

        ui.separator();

        // Build stamp inside the bar itself: whenever the bar is visible you
        // can read which build actually produced it.
        ui.label(
            RichText::new(concat!("v", env!("CARGO_PKG_VERSION")))
                .small()
                .monospace()
                .color(crate::theme::Palette::Dark.colors().text_muted),
        );
    });

    // Expanded section: everything adjustable without pausing playback.
    if app.adjustments_expanded {
        adjustments_body(app, ui);
    }
}

/// In-bar adjustments: visuals (equalizer), speed, scaling and audio, with an
/// explicit Apply Settings button. Lives inside the control bar below the
/// video, so it is always visible and clickable while media plays.
fn adjustments_body(app: &mut NoirApp, ui: &mut Ui) {
    ui.separator();
    ui.label(
        RichText::new(app.tr.t("menu.playback_settings"))
            .strong()
            .small(),
    );

    ui.horizontal(|ui| {
        // Visuals.
        ui.vertical(|ui| {
            ui.label(RichText::new(app.tr.t("menu.visuals")).small());
            {
                let t = &mut app.tweaks;
                ui.add(egui::Slider::new(&mut t.brightness, -100..=100).text("Brightness"));
                ui.add(egui::Slider::new(&mut t.contrast, -100..=100).text("Contrast"));
                ui.add(egui::Slider::new(&mut t.saturation, -100..=100).text("Saturation"));
            }
        });
        ui.vertical(|ui| {
            ui.add_space(18.0);
            {
                let t = &mut app.tweaks;
                ui.add(egui::Slider::new(&mut t.gamma, -100..=100).text("Gamma"));
                ui.add(egui::Slider::new(&mut t.hue, -180..=180).text("Hue"));
            }
        });

        ui.separator();

        // Speed.
        ui.vertical(|ui| {
            ui.label(RichText::new(app.tr.t("menu.speed")).small());
            {
                let t = &mut app.tweaks;
                ui.add(egui::Slider::new(&mut t.speed, 0.25..=4.0).text("x"));
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                for preset in [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0] {
                    if ui.selectable_label(false, format!("{}x", preset)).clicked() {
                        let clamped = app.settings.playback.clamp_speed(preset);
                        app.controller.set_speed(clamped as f64);
                    }
                }
            });
        });
    });

    // Scaling / fill modes apply immediately when clicked.
    ui.horizontal(|ui| {
        ui.label(RichText::new(app.tr.t("menu.scaling")).small());
        let modes = ["Default", "Fill", "16:9", "4:3", "1:1", "2.35:1"];
        for (i, label) in modes.iter().enumerate() {
            if ui.selectable_label(app.fill_mode == i, *label).clicked() {
                app.set_fill_mode(i);
            }
        }
    });

    // Audio.
    let max_volume = match app.settings.playback.max_volume_policy {
        1 => 130u32,
        2 => 200,
        _ => 100,
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new(app.tr.t("menu.audio")).small());
        {
            let t = &mut app.tweaks;
            ui.add(egui::Slider::new(&mut t.volume, 0..=max_volume).text("Volume"));
            ui.checkbox(&mut t.muted, app.tr.t("ctrl.mute"));
        }
        ui.separator();
        if ui.button(app.tr.t("menu.apply")).clicked() {
            app.apply_tweaks();
        }
        if ui.button(app.tr.t("menu.reset")).clicked() {
            app.reset_tweaks();
        }
        if ui.button(app.tr.t("menu.close")).clicked() {
            app.end_adjustments();
        }
    });
}

fn info_strip(ui: &mut Ui, width: f32, height: f32, text: &str) {
    ui.allocate_ui(Vec2::new(width, height), |ui| {
        egui::Frame::default()
            .fill(Color32::from_rgba_unmultiplied(0x09, 0x09, 0x0B, 235))
            .inner_margin(egui::Margin::same(6))
            .corner_radius(CornerRadius::same(4))
            .show(ui, |ui| {
                ui.set_max_width(340.0);
                ui.label(RichText::new(text).monospace().small());
            });
    });
}

fn info_lines(app: &NoirApp, snapshot: &EngineSnapshot) -> Vec<String> {
    let stats = &snapshot.stats;
    let mut lines = Vec::new();
    if let (Some(w), Some(h)) = (stats.width, stats.height) {
        lines.push(format!("{}x{}", w, h));
    }
    if let Some(codec) = &stats.video_codec {
        lines.push(format!("V: {codec}"));
    }
    if let Some(fps) = stats.fps {
        lines.push(format!("{fps:.2} fps"));
    }
    if let Some(br) = stats.video_bitrate {
        lines.push(format!("{} kbps", br as u64 / 1000));
    }
    if let Some(codec) = &stats.audio_codec {
        lines.push(format!("A: {codec}"));
    }
    if let Some(hw) = &stats.hw_decoder {
        lines.push(format!("HW: {hw}"));
    } else if app.capabilities.hardware_decoding {
        lines.push("HW: off".into());
    }
    if let Some(dropped) = stats.dropped_frames {
        lines.push(format!("Dropped: {dropped}"));
    }
    if let Some(sync) = stats.av_sync {
        lines.push(format!("A/V: {sync:+.3}s"));
    }
    lines.push(format!("Speed: {:.2}x", snapshot.speed));
    if let Some(audio) = snapshot
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Audio && t.selected)
    {
        lines.push(format!("Audio: {}", track_label(audio)));
    }
    if let Some(sub) = snapshot
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Subtitle && t.selected)
    {
        lines.push(format!("Sub: {}", track_label(sub)));
    }
    lines
}

fn track_label(t: &TrackInfo) -> String {
    let mut parts = vec![t.id.to_string()];
    if let Some(lang) = &t.lang {
        parts.push(lang.clone());
    }
    if let Some(title) = &t.title {
        parts.push(title.clone());
    }
    parts.join(" — ")
}

/// Custom monochrome timeline: thin bar with a circular thumb, click and drag
/// to seek.
fn timeline(
    ui: &mut Ui,
    position: f64,
    duration: f64,
    scrub: &mut Option<f64>,
    scrubbing: &mut Option<f64>,
) {
    let desired = Vec2::new(ui.available_width() - 40.0, 14.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
    let bar = Rect::from_min_size(
        egui::pos2(rect.left(), rect.center().y - 2.5),
        Vec2::new(rect.width(), 5.0),
    );
    let colors = crate::theme::Palette::Dark.colors();
    ui.painter()
        .rect_filled(bar, CornerRadius::same(2), colors.surface_secondary);
    let fraction = if duration > 0.0 {
        (position / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if fraction > 0.0 {
        let fill = Rect::from_min_size(
            bar.min,
            Vec2::new(bar.width() * fraction as f32, bar.height()),
        );
        ui.painter()
            .rect_filled(fill, CornerRadius::same(2), colors.highlight);
    }
    let knob_x = bar.left() + bar.width() * fraction as f32;
    ui.painter().circle_filled(
        egui::pos2(knob_x, bar.center().y),
        5.5,
        if response.hovered() || response.dragged() {
            colors.highlight
        } else {
            colors.text_secondary
        },
    );

    let frac_from_pointer = |pos: Option<egui::Pos2>| -> f64 {
        let x = pos.unwrap_or(bar.min).x;
        (((x - bar.left()) / bar.width().max(1.0)) as f64).clamp(0.0, 1.0)
    };
    if response.dragged() || response.clicked() {
        let target = frac_from_pointer(response.interact_pointer_pos()) * duration;
        *scrub = Some(target);
        *scrubbing = Some(target);
    }
}

/// Format seconds as H:MM:SS or M:SS.
pub fn format_time(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "--:--".into();
    }
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

pub fn state_label(app: &NoirApp, state: &PlaybackState) -> String {
    match state {
        PlaybackState::Idle => app.tr.t("state.idle"),
        PlaybackState::Loading => app.tr.t("state.loading"),
        PlaybackState::Playing => app.tr.t("state.playing"),
        PlaybackState::Paused => app.tr.t("state.paused"),
        PlaybackState::Buffering => app.tr.t("state.buffering"),
        PlaybackState::Ended => app.tr.t("state.ended"),
        PlaybackState::Error(_) => app.tr.t("state.error"),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_times() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(59.4), "0:59");
        assert_eq!(format_time(65.0), "1:05");
        assert_eq!(format_time(3661.0), "1:01:01");
        assert_eq!(format_time(-3.0), "--:--");
        assert_eq!(format_time(f64::NAN), "--:--");
    }
}

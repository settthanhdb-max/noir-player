//! Player screen: video viewport, timeline, control panel and info overlay.

use crate::app::NoirApp;
use crate::theme::Palette;
use egui::{Align2, Color32, CornerRadius, Rect, RichText, Sense, Ui, Vec2};
use noir_core::models::{TrackInfo, TrackKind};
use noir_core::state::PlaybackState;

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    let appearance = app.settings.appearance.clone();
    let snapshot = app.controller.snapshot();
    let controls_visible = app.controls_visible(&snapshot);

    let (viewport_rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());

    // Viewport background: pure black so letterboxing is seamless.
    ui.painter().rect_filled(viewport_rect, 0, Color32::BLACK);

    if snapshot.current_path.is_none() {
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

    // Viewport interactions.
    let interaction = ui.interact(
        viewport_rect,
        egui::Id::unique("video_viewport"),
        Sense::click_and_drag(),
    );
    if interaction.hovered() {
        app.mark_mouse_activity();
    }
    if interaction.clicked() {
        app.controller.toggle_play();
        app.mark_mouse_activity();
    }
    if interaction.double_clicked() {
        app.toggle_fullscreen(ui.ctx());
        app.mark_mouse_activity();
    }

    // Information overlay.
    if app.show_info && appearance.show_info_overlay {
        info_overlay(app, ui.ctx(), viewport_rect, &snapshot);
    }

    // Control panel overlay at the bottom of the viewport.
    if controls_visible && appearance.show_controls {
        egui::Area::new(egui::Id::unique("player_controls"))
            .anchor(Align2::CENTER_BOTTOM, [0.0, -4.0])
            .order(egui::Order::Middle)
            .show(ui.ctx(), |ui| {
                egui::Frame::default()
                    .fill(Palette::Dark.colors().surface)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .stroke(egui::Stroke::new(1.0, Palette::Dark.colors().border))
                    .show(ui, |ui| control_panel(app, ui, &snapshot));
            });
    }
}

fn info_overlay(
    app: &NoirApp,
    ctx: &egui::Context,
    _viewport: Rect,
    snapshot: &noir_playback::engine::EngineSnapshot,
) {
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

    let anchor = match app.settings.appearance.osd_position {
        0 => (Align2::LEFT_TOP, [8.0, 8.0]),
        1 => (Align2::RIGHT_TOP, [-8.0, 8.0]),
        2 => (Align2::LEFT_BOTTOM, [8.0, -8.0]),
        _ => (Align2::RIGHT_BOTTOM, [-8.0, -8.0]),
    };
    let text = lines.join("\n");
    egui::Area::new(egui::Id::unique("info_overlay"))
        .anchor(anchor.0, anchor.1)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::default()
                .fill(Color32::from_rgba_unmultiplied(0x09, 0x09, 0x0B, 220))
                .inner_margin(egui::Margin::same(6))
                .corner_radius(CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.set_max_width(240.0);
                    ui.label(RichText::new(text).monospace().small());
                });
        });
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

fn control_panel(app: &mut NoirApp, ui: &mut Ui, snapshot: &noir_playback::engine::EngineSnapshot) {
    let playback = app.settings.playback.clone();
    let max_volume = match app.settings.playback.max_volume_policy {
        1 => 130u32,
        2 => 200,
        _ => 100,
    };

    // Timeline row.
    let duration = snapshot.duration.max(0.0);
    let position = app.scrub_position.unwrap_or(snapshot.position);
    let mut scrub = app.scrub_position;
    timeline(ui, position, duration, &mut scrub, &mut app.scrubbing);
    app.scrub_position = scrub;
    if let Some(target) = app.scrubbing {
        let primary_down = ui.input(|i| i.pointer.primary_down());
        if !primary_down {
            app.controller
                .execute(noir_playback::engine::EngineCommand::SeekAbsolute { position: target });
            app.scrubbing = None;
            app.scrub_position = None;
        }
    }

    // Buttons row.
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

        ui.monospace(format_time(position));
        ui.label(RichText::new("/").color(Color32::from_rgb(0x71, 0x71, 0x7A)));
        ui.monospace(format_time(duration));

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
        ui.label(RichText::new(snapshot.volume.to_string()).small());

        ui.separator();

        if ui.button(format!("{:.2}x", snapshot.speed)).clicked() {
            app.speed_menu_open = !app.speed_menu_open;
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
    });

    // Speed presets, shown on demand under the speed button.
    if app.speed_menu_open {
        ui.horizontal(|ui| {
            for preset in [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0] {
                if ui.small_button(format!("{preset:.2}x")).clicked() {
                    app.controller
                        .set_speed(playback.clamp_speed(preset) as f64);
                    app.speed_menu_open = false;
                }
            }
        });
    }
}

/// Custom monochrome timeline: thin bar, click and drag to seek.
fn timeline(
    ui: &mut Ui,
    position: f64,
    duration: f64,
    scrub: &mut Option<f64>,
    scrubbing: &mut Option<f64>,
) {
    let desired = Vec2::new(ui.available_width(), 14.0);
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
        4.5,
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

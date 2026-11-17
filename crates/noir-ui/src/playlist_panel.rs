//! Collapsible queue panel: search, reorder, remove, save/load, repeat/shuffle.

use crate::app::NoirApp;
use crate::player::format_time;
use crate::state::RepeatMode;
use egui::Ui;

pub fn panel(app: &mut NoirApp, ui: &mut Ui) {
    let width = app.settings.appearance.playlist_width.max(200.0);
    ui.set_min_width(width);

    ui.horizontal(|ui| {
        ui.strong(app.tr.t("pl.queue"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(app.tr.t("pl.clear")).clicked() {
                app.queue.clear();
                app.queue_index = None;
            }
            if ui.small_button(app.tr.t("pl.save")).clicked() {
                app.save_queue_prompt = true;
            }
            if ui.small_button(app.tr.t("pl.import_m3u")).clicked() {
                app.dialogs.request(
                    crate::dialogs::DialogKind::OpenM3u,
                    Some("m3u-import".into()),
                );
            }
            if ui.small_button(app.tr.t("pl.export_m3u")).clicked() {
                let paths: Vec<String> = app.queue.iter().map(|e| e.path.clone()).collect();
                app.dialogs.request(
                    crate::dialogs::DialogKind::SaveM3u,
                    Some(serde_json::to_string(&paths).unwrap_or_default()),
                );
            }
        });
    });
    ui.separator();

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.queue_search)
                .hint_text(app.tr.t("lib.search"))
                .desired_width(ui.available_width()),
        );
    });

    ui.horizontal(|ui| {
        let repeat = RepeatMode::from_index(app.settings.playlist.repeat_mode);
        let repeat_label = match repeat {
            RepeatMode::Off => app.tr.t("pl.repeat_off"),
            RepeatMode::One => app.tr.t("pl.repeat_one"),
            RepeatMode::All => app.tr.t("pl.repeat_all"),
        };
        if ui
            .selectable_label(repeat != RepeatMode::Off, repeat_label)
            .clicked()
        {
            let next = match repeat {
                RepeatMode::Off => 2,
                RepeatMode::All => 1,
                RepeatMode::One => 0,
            };
            app.settings.playlist.repeat_mode = next;
            app.on_settings_changed();
        }
        if ui
            .selectable_label(app.settings.playlist.shuffle, app.tr.t("pl.shuffle"))
            .clicked()
        {
            app.settings.playlist.shuffle = !app.settings.playlist.shuffle;
            app.on_settings_changed();
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{}", app.queue.len()))
                    .small()
                    .color(egui::Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
        });
    });
    ui.separator();

    let search = app.queue_search.to_ascii_lowercase();
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            if app.queue.is_empty() {
                ui.label(
                    egui::RichText::new(app.tr.t("pl.empty"))
                        .color(egui::Color32::from_rgb(0x71, 0x71, 0x7A)),
                );
                return;
            }
            let snapshot = app.controller.snapshot();
            let current_path = snapshot.current_path.clone();
            for i in 0..app.queue.len() {
                let entry = app.queue[i].clone();
                if !search.is_empty() && !entry.title.to_ascii_lowercase().contains(&search) {
                    continue;
                }
                let is_current = current_path.as_deref() == Some(entry.path.as_str());
                let mut remove = false;
                let mut play = false;
                ui.horizontal(|ui| {
                    if is_current {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(12.0, ui.available_height()),
                            egui::Sense::hover(),
                        );
                        crate::icons::paint(
                            ui.painter(),
                            crate::icons::Icon::Play,
                            rect,
                            ui.style().visuals.strong_text_color(),
                        );
                    }
                    let title = entry.title.clone();
                    let resp = ui
                        .selectable_label(is_current, title)
                        .on_hover_text(&entry.path);
                    if resp.clicked() {
                        play = true;
                    }
                    if resp.double_clicked() {
                        play = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if crate::icons::icon_button_small(
                            ui,
                            crate::icons::Icon::Close,
                            app.tr.t("queue.remove"),
                        ) {
                            remove = true;
                        }
                        if app.settings.playlist.show_duration {
                            ui.label(
                                egui::RichText::new(
                                    entry
                                        .duration
                                        .map(format_time)
                                        .unwrap_or_else(|| "--:--".into()),
                                )
                                .small(),
                            );
                        }
                        if app.settings.playlist.show_media_type {
                            ui.label(
                                egui::RichText::new(entry.kind.label())
                                    .small()
                                    .color(egui::Color32::from_rgb(0xA1, 0xA1, 0xAA)),
                            );
                        }
                    });
                });
                // Reorder controls on hover row.
                ui.horizontal(|ui| {
                    if crate::icons::icon_button_small(ui, crate::icons::Icon::Up, "Move up")
                        && i > 0
                    {
                        app.queue.swap(i - 1, i);
                        if app.queue_index == Some(i) {
                            app.queue_index = Some(i - 1);
                        }
                    }
                    if crate::icons::icon_button_small(ui, crate::icons::Icon::Down, "Move down")
                        && i + 1 < app.queue.len()
                    {
                        app.queue.swap(i, i + 1);
                        if app.queue_index == Some(i) {
                            app.queue_index = Some(i + 1);
                        }
                    }
                    ui.add_space(4.0);
                });
                ui.separator();
                if play {
                    app.play_from_queue(i);
                }
                if remove {
                    app.queue.remove(i);
                    if app.queue_index == Some(i) {
                        app.queue_index = None;
                    } else if let Some(idx) = app.queue_index {
                        if idx > i {
                            app.queue_index = Some(idx - 1);
                        }
                    }
                }
            }
        });

    if app.save_queue_prompt {
        egui::Window::new(app.tr.t("pl.save"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.label(app.tr.t("pl.name_prompt"));
                ui.add(egui::TextEdit::singleline(&mut app.queue_name_input).desired_width(240.0));
                ui.horizontal(|ui| {
                    if ui.button(app.tr.t("misc.ok")).clicked() {
                        let name = app.queue_name_input.trim().to_string();
                        if !name.is_empty() {
                            let paths: Vec<String> =
                                app.queue.iter().map(|e| e.path.clone()).collect();
                            if let Some(lib) = &app.library {
                                lib.save_playlist(&name, &paths);
                            }
                        }
                        app.save_queue_prompt = false;
                        app.queue_name_input.clear();
                    }
                    if ui.button(app.tr.t("misc.cancel")).clicked() {
                        app.save_queue_prompt = false;
                    }
                });
            });
    }
}

//! Home screen: continue watching, recents, favorites, playlists, quick open.

use crate::app::NoirApp;
use crate::player::format_time;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading(app.tr.t("home.title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(app.tr.t("home.quick_open")).clicked() {
                    app.dialogs.request(
                        crate::dialogs::DialogKind::OpenMedia,
                        Some("open-media".into()),
                    );
                }
                if ui.button(app.tr.t("ctrl.open_folder")).clicked() {
                    app.dialogs.request(
                        crate::dialogs::DialogKind::PickDirectory,
                        Some("open-folder".into()),
                    );
                }
            });
        });
        ui.separator();

        let Some(lib) = app.library.clone() else {
            ui.label(
                RichText::new(app.tr.t("home.empty")).color(Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
            return;
        };

        let history = lib.history(50);
        let continue_watching: Vec<_> = history
            .iter()
            .filter(|h| h.position > 1.0 && h.completion < 0.95)
            .take(8)
            .cloned()
            .collect();
        if !continue_watching.is_empty() {
            section(app, ui, app.tr.t("home.continue_watching"), |ui, app| {
                for entry in &continue_watching {
                    row(
                        app,
                        ui,
                        &entry.title,
                        &entry.path,
                        Some(&format!(
                            "{} / {} — {}%",
                            format_time(entry.position),
                            entry
                                .duration
                                .map(format_time)
                                .unwrap_or_else(|| "--:--".into()),
                            (entry.completion * 100.0) as u32
                        )),
                    );
                }
            });
        }

        let recent = lib.recent_media(8);
        if !recent.is_empty() {
            let recent = recent.clone();
            section(app, ui, app.tr.t("home.recent"), |ui, app| {
                for item in &recent {
                    row(
                        app,
                        ui,
                        &item.title,
                        &item.path.to_string_lossy(),
                        Some(&recent_line(item)),
                    );
                }
            });
        }

        let frequent = lib.frequently_played(8);
        if !frequent.is_empty() {
            let frequent = frequent.clone();
            section(app, ui, app.tr.t("home.frequent"), |ui, app| {
                for item in &frequent {
                    row(
                        app,
                        ui,
                        &item.title,
                        &item.path.to_string_lossy(),
                        Some(&recent_line(item)),
                    );
                }
            });
        }

        let favorites = lib.favorites();
        if !favorites.is_empty() {
            let favorites = favorites.clone();
            section(app, ui, app.tr.t("home.favorites"), |ui, app| {
                for item in &favorites {
                    row(
                        app,
                        ui,
                        &item.title,
                        &item.path.to_string_lossy(),
                        Some(&recent_line(item)),
                    );
                }
            });
        }

        let playlists = lib.playlists();
        if !playlists.is_empty() {
            let playlists = playlists.clone();
            section(app, ui, app.tr.t("home.playlists"), |ui, app| {
                for pl in &playlists {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(false, &pl.name).clicked() {
                            app.load_playlist_into_queue(&pl.item_paths);
                        }
                        ui.label(
                            RichText::new(format!(
                                "{} {}",
                                pl.item_paths.len(),
                                app.tr.t("lib.files")
                            ))
                            .small()
                            .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
                        );
                    });
                }
            });
        }

        if history.is_empty() && recent.is_empty() && favorites.is_empty() {
            ui.add_space(24.0);
            ui.label(
                RichText::new(app.tr.t("home.empty")).color(Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
        }
    });
}

fn recent_line(item: &noir_core::models::MediaItem) -> String {
    let mut parts = Vec::new();
    parts.push(item.kind.label().to_string());
    if let Some(d) = item.duration {
        parts.push(format_time(d));
    }
    if item.play_count > 0 {
        parts.push(format!("×{}", item.play_count));
    }
    parts.join(" · ")
}

fn section(
    app: &mut NoirApp,
    ui: &mut Ui,
    title: &str,
    content: impl FnOnce(&mut Ui, &mut NoirApp),
) {
    ui.add_space(8.0);
    ui.strong(title);
    ui.indent(ui.scope_id().with(title), |ui| {
        content(ui, app);
    });
}

fn row(app: &mut NoirApp, ui: &mut Ui, title: &str, path: &str, meta: Option<&str>) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(false, title)
            .on_hover_text(path)
            .clicked()
        {
            app.open_and_play(path.to_string());
        }
        if let Some(meta) = meta {
            ui.label(
                RichText::new(meta)
                    .small()
                    .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
        }
    });
}

//! Playlist Manager screen: saved playlists, queue editing, M3U import/export.

use crate::app::NoirApp;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    ui.heading(app.tr.t("pl.title"));
    ui.separator();

    ui.columns(2, |columns| {
        // Saved playlists.
        columns[0].vertical(|ui| {
            ui.strong(app.tr.t("pl.saved"));
            ui.separator();
            let Some(lib) = app.library.clone() else {
                return;
            };
            let mut delete: Option<i64> = None;
            let mut load: Option<Vec<String>> = None;
            ScrollArea::vertical().show(ui, |ui| {
                for pl in lib.playlists() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(false, &pl.name).clicked() {
                            load = Some(pl.item_paths.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(app.tr.t("pl.delete")).clicked() {
                                delete = Some(pl.id);
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
                    });
                }
            });
            if let Some(id) = delete {
                lib.delete_playlist(id);
            }
            if let Some(paths) = load {
                app.load_playlist_into_queue(&paths);
                app.screen = crate::state::Screen::Player;
            }
        });

        // Current queue.
        columns[1].vertical(|ui| {
            ui.strong(app.tr.t("pl.queue"));
            ui.separator();
            crate::playlist_panel::panel(app, ui);
        });
    });
}

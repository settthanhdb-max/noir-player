//! Media Library screen: directories, scanning, search, sort, filter, favorites.

use crate::app::NoirApp;
use crate::player::format_time;
use egui::{Color32, RichText, ScrollArea, Ui};
use noir_core::models::{MediaItem, MediaKind};

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    let Some(lib) = app.library.clone() else {
        ui.label(RichText::new(app.tr.t("lib.empty")).color(Color32::from_rgb(0x71, 0x71, 0x7A)));
        return;
    };

    ui.horizontal(|ui| {
        ui.heading(app.tr.t("lib.title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(app.tr.t("lib.add_dir")).clicked() && !app.dialogs.is_busy() {
                app.dialogs.request(
                    crate::dialogs::DialogKind::PickDirectory,
                    Some("lib-add-dir".into()),
                );
            }
            if ui.button(app.tr.t("lib.rescan")).clicked() {
                app.rescan_all();
            }
            if ui.button(app.tr.t("lib.remove_missing")).clicked() {
                lib.remove_missing();
            }
        });
    });

    // Scan status line.
    if let Some(status) = &app.scan_status {
        ui.label(
            RichText::new(status)
                .small()
                .color(Color32::from_rgb(0xA1, 0xA1, 0xAA)),
        );
    }

    // Directories.
    ui.strong(app.tr.t("lib.directories"));
    ui.indent("dirs", |ui| {
        for dir in lib.directories() {
            ui.horizontal(|ui| {
                ui.monospace(&dir);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(app.tr.t("lib.remove_dir")).clicked() {
                        lib.remove_directory(&dir);
                    }
                    if ui.small_button(app.tr.t("lib.rescan")).clicked() {
                        app.scan_rx = Some(lib.scan_directory(dir.clone().into()));
                    }
                });
            });
        }
    });
    ui.separator();

    // Search + filter + sort.
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.library_search)
                .hint_text(app.tr.t("lib.search"))
                .desired_width(240.0),
        );
        let mut filter = app.library_filter;
        egui::ComboBox::from_id_salt("lib_filter")
            .selected_text(filter_label(filter))
            .show_ui(ui, |ui| {
                for i in 0..4 {
                    ui.selectable_value(&mut filter, i, filter_label(i));
                }
            });
        app.library_filter = filter;

        let mut sort = app.library_sort;
        egui::ComboBox::from_id_salt("lib_sort")
            .selected_text(sort_label(sort))
            .show_ui(ui, |ui| {
                for i in 0..5 {
                    ui.selectable_value(&mut sort, i, sort_label(i));
                }
            });
        app.library_sort = sort;
    });
    ui.separator();

    let mut items = if app.library_search.is_empty() {
        lib.all_media()
    } else {
        lib.search(&app.library_search)
    };
    items.sort_by(|a, b| match app.library_sort {
        1 => b.added_at.cmp(&a.added_at),
        2 => a
            .duration
            .partial_cmp(&b.duration)
            .unwrap_or(std::cmp::Ordering::Equal),
        3 => a
            .kind
            .label()
            .cmp(b.kind.label())
            .then(a.title.cmp(&b.title)),
        4 => b.last_played_at.cmp(&a.last_played_at),
        _ => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
    });
    items.retain(|item| match app.library_filter {
        1 => item.kind == MediaKind::Video,
        2 => item.kind == MediaKind::Audio,
        3 => item.favorite,
        _ => true,
    });

    ui.label(
        RichText::new(format!("{} {}", items.len(), app.tr.t("lib.files")))
            .small()
            .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
    );

    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        if items.is_empty() {
            ui.label(
                RichText::new(app.tr.t("lib.empty")).color(Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
        }
        for item in items {
            media_row(app, ui, &item);
        }
    });
}

fn filter_label(index: usize) -> &'static str {
    match index {
        1 => "Video",
        2 => "Audio",
        3 => "Favorites",
        _ => "All",
    }
}

fn sort_label(index: usize) -> &'static str {
    match index {
        1 => "Date added",
        2 => "Duration",
        3 => "File type",
        4 => "Last played",
        _ => "Name",
    }
}

fn media_row(app: &mut NoirApp, ui: &mut Ui, item: &MediaItem) {
    let mut queued = false;
    ui.horizontal(|ui| {
        let path = item.path.to_string_lossy().to_string();
        let resp = ui.selectable_label(false, &item.title).on_hover_text(&path);
        if resp.clicked() {
            app.open_and_play(path.clone());
        }
        if resp.double_clicked() {
            app.open_and_play(path.clone());
        }
        if ui
            .small_button("+")
            .on_hover_text(app.tr.t("ctrl.playlist"))
            .clicked()
        {
            queued = true;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let star = if item.favorite { "★" } else { "☆" };
            if ui.small_button(star).clicked() {
                lib_toggle_favorite(app, &path, !item.favorite);
            }
            if app.settings.playlist.show_duration {
                ui.label(
                    RichText::new(
                        item.duration
                            .map(format_time)
                            .unwrap_or_else(|| "--:--".into()),
                    )
                    .small(),
                );
            }
            if app.settings.playlist.show_file_size {
                if let Some(size) = item.size_bytes {
                    ui.label(
                        RichText::new(noir_platform::filesystem::format_size(size))
                            .small()
                            .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
                    );
                }
            }
            ui.label(
                RichText::new(item.kind.label())
                    .small()
                    .color(Color32::from_rgb(0xA1, 0xA1, 0xAA)),
            );
        });
    });
    if queued {
        app.queue.push(crate::state::QueueEntry::from_media(item));
    }
}

fn lib_toggle_favorite(app: &mut NoirApp, path: &str, favorite: bool) {
    if let Some(lib) = &app.library {
        lib.set_favorite(std::path::Path::new(path), favorite);
    }
}

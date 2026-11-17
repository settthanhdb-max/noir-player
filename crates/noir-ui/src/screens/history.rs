//! Playback History screen.

use crate::app::NoirApp;
use crate::player::format_time;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.heading(app.tr.t("hist.title"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(app.tr.t("hist.clear")).clicked() {
                if app.settings.general.confirm_clear_history {
                    app.confirm_clear_history = true;
                } else if let Some(lib) = &app.library {
                    lib.clear_history();
                }
            }
        });
    });
    if !app.settings.playback.record_history {
        ui.label(
            RichText::new(app.tr.t("hist.recording"))
                .small()
                .color(Color32::from_rgb(0xA1, 0xA1, 0xAA)),
        );
    }
    ui.separator();

    if app.confirm_clear_history {
        ui.horizontal(|ui| {
            ui.label(app.tr.t("set.reset_confirm"));
            if ui.button(app.tr.t("set.yes")).clicked() {
                if let Some(lib) = &app.library {
                    lib.clear_history();
                }
                app.confirm_clear_history = false;
            }
            if ui.button(app.tr.t("set.no")).clicked() {
                app.confirm_clear_history = false;
            }
        });
        ui.separator();
    }

    let Some(lib) = app.library.clone() else {
        return;
    };
    let history = lib.history(500);
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        if history.is_empty() {
            ui.label(
                RichText::new(app.tr.t("hist.empty")).color(Color32::from_rgb(0x71, 0x71, 0x7A)),
            );
        }
        for entry in history {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(false, &entry.title)
                    .on_hover_text(&entry.path)
                    .clicked()
                {
                    let resume = if app.settings.playback.auto_resume
                        && entry.position > app.settings.playback.resume_threshold_s as f64
                    {
                        Some(entry.position)
                    } else {
                        None
                    };
                    app.controller.open(entry.path.clone(), resume);
                    app.screen = crate::state::Screen::Player;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} / {} · {}%",
                            format_time(entry.position),
                            entry
                                .duration
                                .map(format_time)
                                .unwrap_or_else(|| "--:--".into()),
                            (entry.completion * 100.0) as u32
                        ))
                        .small()
                        .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
                    );
                });
            });
        }
    });
}

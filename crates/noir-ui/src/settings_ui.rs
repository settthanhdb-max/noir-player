//! Settings screen, generated from the descriptor system in `noir-settings`.
//!
//! Every row is rendered from a [`SettingEntry`]; adding a setting to the
//! schema automatically adds it here, with search, tooltips, capability
//! gating, advanced filtering and reset support.

use crate::app::NoirApp;
use crate::i18n::Tr;
use crate::state::PickRequest;
use egui::{Color32, RichText, ScrollArea, Ui};
use noir_playback::Capabilities;
use noir_settings::schema::input::KeyBinding;
use noir_settings::schema::Settings;
use noir_settings::value::{SettingEntry, SettingKind, SettingValue};

pub fn screen(app: &mut NoirApp, ui: &mut Ui) {
    let tr = app.tr;
    let mut search = app.settings_search.clone();
    let mut selected = app.settings_category;
    let mut show_advanced = app.show_advanced;
    let mut picks: Vec<PickRequest> = Vec::new();
    let mut changed = false;
    let mut reset_category: Option<String> = None;
    let mut reset_all = false;
    let mut apply_settings = false;
    let mut import_cfg = false;
    let mut export_cfg = false;
    let mut import_keys = false;
    let mut export_keys = false;
    let mut capture = app.capture_action;
    let mut conflicts_report = String::new();
    let mut bindings_json =
        serde_json::to_string(&app.settings.input.bindings).unwrap_or_else(|_| "[]".into());

    // `describe_all` borrows the settings mutably, so the whole render pass
    // works on local copies of app UI state, which are written back below.
    {
        let mut categories = app.settings.describe_all();
        let category_names: Vec<&'static str> = categories.iter().map(|(n, _)| *n).collect();

        egui::Panel::left("settings_sidebar")
            .resizable(false)
            .default_size(190.0)
            .show(ui, |ui| {
                ui.add_space(6.0);
                ui.add(
                    egui::TextEdit::singleline(&mut search)
                        .hint_text(tr.t("set.search"))
                        .desired_width(ui.available_width()),
                );
                ui.add_space(4.0);
                ui.checkbox(&mut show_advanced, tr.t("set.advanced"));
                ui.separator();
                ui.label(
                    RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .small()
                        .color(Color32::from_rgb(0x71, 0x71, 0x7A)),
                );
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, name) in category_names.iter().enumerate() {
                        let label = category_label(name, &tr);
                        if !search.is_empty() && !search_matches(&categories, i, &search) {
                            continue;
                        }
                        if ui.selectable_label(selected == i, label).clicked() {
                            selected = i;
                        }
                    }
                });
                ui.separator();
                if ui.button(tr.t("set.import")).clicked() {
                    import_cfg = true;
                }
                if ui.button(tr.t("set.export")).clicked() {
                    export_cfg = true;
                }
                if ui.button(tr.t("set.reset_all")).clicked() {
                    reset_all = true;
                }
                if ui.button(tr.t("set.apply")).clicked() {
                    apply_settings = true;
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(crate::theme::Palette::Dark.colors().background))
            .show(ui, |ui| {
                let Some((name, entries)) = categories.get_mut(selected) else {
                    return;
                };
                ui.horizontal(|ui| {
                    ui.heading(category_label(name, &tr));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(tr.t("set.reset_category")).clicked() {
                            reset_category = Some((*name).to_string());
                        }
                    });
                });
                ui.separator();

                let caps = app.capabilities.clone();
                ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    let filtering = !search.is_empty();
                    for entry in entries.iter_mut() {
                        if filtering && !entry_matches(entry, &search) {
                            continue;
                        }
                        render_entry(
                            ui,
                            entry,
                            &tr,
                            &caps,
                            show_advanced,
                            &mut picks,
                            &mut changed,
                        );
                    }
                    if *name == "input" {
                        ui.add_space(12.0);
                        keyboard_editor(
                            ui,
                            &tr,
                            &mut bindings_json,
                            &mut capture,
                            &mut conflicts_report,
                            &mut import_keys,
                            &mut export_keys,
                        );
                    }
                });
            });
    }

    if let Ok(bindings) = serde_json::from_str::<Vec<KeyBinding>>(&bindings_json) {
        app.settings.input.bindings = bindings;
    }
    app.settings_search = search;
    app.settings_category = selected;
    app.show_advanced = show_advanced;
    app.capture_action = capture;

    for pick in picks {
        match pick {
            PickRequest::Directory { key } => {
                app.pending_pick = Some(PickRequest::Directory { key });
                app.dialogs
                    .request(crate::dialogs::DialogKind::PickDirectory, None);
            }
            PickRequest::File { key } => {
                app.pending_pick = Some(PickRequest::File { key });
                app.dialogs
                    .request(crate::dialogs::DialogKind::OpenJson, None);
            }
            _ => {}
        }
    }

    if import_cfg {
        app.dialogs.request(
            crate::dialogs::DialogKind::OpenJson,
            Some("config-import".into()),
        );
    }
    if export_cfg {
        app.dialogs.request(
            crate::dialogs::DialogKind::SaveJson,
            Some("config-export".into()),
        );
    }
    if import_keys {
        app.dialogs.request(
            crate::dialogs::DialogKind::OpenJson,
            Some("keys-import".into()),
        );
    }
    if export_keys {
        app.dialogs.request(
            crate::dialogs::DialogKind::SaveJson,
            Some("keys-export".into()),
        );
    }
    if reset_all {
        let defaults = Settings::defaults();
        app.settings = defaults;
        changed = true;
        app.settings_category = 0;
    }
    if let Some(name) = reset_category {
        app.reset_category(&name);
        changed = true;
    }
    if !conflicts_report.is_empty() {
        app.osd_message = Some((conflicts_report, std::time::Instant::now()));
    }
    if changed {
        app.on_settings_changed();
    }
    if apply_settings {
        // Re-validate, persist and push the live settings (language, theme,
        // engine commands) so every change takes effect immediately.
        app.on_settings_changed();
        app.osd_message = Some((
            app.tr.t("set.applied").to_string(),
            std::time::Instant::now(),
        ));
    }
}

fn category_label(name: &str, tr: &Tr) -> String {
    match name {
        "general" => "General".to_string(),
        "appearance" => "Appearance".to_string(),
        "playback" => "Playback".to_string(),
        "video" => "Video".to_string(),
        "audio" => "Audio".to_string(),
        "subtitles" => "Subtitles".to_string(),
        "network" => "Network".to_string(),
        "playlist" => "Playlist".to_string(),
        "input" => "Keyboard & Mouse".to_string(),
        "advanced" => "Advanced".to_string(),
        other => tr.t(other).to_string(),
    }
}

fn search_matches(
    categories: &[(&'static str, Vec<SettingEntry<'_>>)],
    index: usize,
    search: &str,
) -> bool {
    categories[index].1.iter().any(|e| entry_matches(e, search))
}

fn entry_matches(entry: &SettingEntry<'_>, search: &str) -> bool {
    let s = search.to_ascii_lowercase();
    entry.label.to_ascii_lowercase().contains(&s)
        || entry.description.to_ascii_lowercase().contains(&s)
        || entry.key.contains(&s)
}

fn cap_ok(caps: &Capabilities, capability: &str) -> bool {
    match capability {
        "" => true,
        "hwdec" => caps.hardware_decoding,
        "network" => caps.network_streaming,
        "audio_dsp" => caps.audio_dsp,
        "video_eq" => caps.video_equalizer,
        "screenshot" => caps.screenshot,
        "ab_repeat" => caps.ab_repeat,
        "passthrough" => caps.audio && caps.audio_dsp,
        "hdr" => caps.video && caps.hardware_decoding,
        other => other == "always",
    }
}

fn render_entry(
    ui: &mut Ui,
    entry: &mut SettingEntry<'_>,
    tr: &Tr,
    caps: &Capabilities,
    show_advanced: bool,
    picks: &mut Vec<PickRequest>,
    changed: &mut bool,
) {
    if entry.advanced && !show_advanced {
        return;
    }
    let supported = cap_ok(caps, entry.requires_capability);
    let default_display = entry.kind.default_display();
    let modified = !entry.value.is_default(&default_display);

    ui.horizontal(|ui| {
        ui.set_min_width(ui.available_width());
        ui.vertical(|ui| {
            let mut label = RichText::new(entry.label);
            if !supported {
                label = label.color(ui.visuals().widgets.noninteractive.bg_stroke.color);
            } else if modified {
                label = label.strong();
            }
            let response = ui.label(label).on_hover_text(entry.description);
            if entry.restart_required {
                response.on_hover_text(format!(
                    "{} — {}",
                    tr.t("set.restart_required"),
                    entry.description
                ));
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let row_changed = render_control(ui, entry, picks);
            if row_changed {
                *changed = true;
            }
            if entry.restart_required {
                ui.label(RichText::new("*").color(Color32::from_rgb(0xA1, 0xA1, 0xAA)))
                    .on_hover_text(tr.t("set.restart_required"));
            }
            if !supported {
                ui.label(RichText::new(tr.t("set.needs_capability")).small());
            } else if modified && ui.small_button(tr.t("set.reset")).clicked() {
                reset_one(entry, &default_display);
                *changed = true;
            }
        });
    });
    ui.add_space(2.0);
}

fn render_control(ui: &mut Ui, entry: &mut SettingEntry<'_>, picks: &mut Vec<PickRequest>) -> bool {
    if !entry.requires_capability.is_empty() {
        ui.disable();
    }
    let mut changed = false;
    match (&mut entry.value, &entry.kind) {
        (SettingValue::Bool(v), _) => {
            changed = ui.checkbox(v, "").changed();
        }
        (SettingValue::U32(v), SettingKind::Number { min, max, .. }) => {
            let mut as_f = **v as f64;
            changed = ui
                .add(
                    egui::DragValue::new(&mut as_f)
                        .range(*min..=*max)
                        .speed(1.0),
                )
                .changed();
            **v = as_f.round() as u32;
        }
        (SettingValue::U32(v), SettingKind::Slider { min, max, unit, .. }) => {
            let mut as_f = **v as f64;
            changed = ui
                .add(egui::Slider::new(&mut as_f, *min..=*max).suffix(*unit))
                .changed();
            **v = as_f.round() as u32;
        }
        (SettingValue::U32(v), SettingKind::Dropdown { options }) => {
            let idx = (**v as usize).min(options.len().saturating_sub(1));
            let mut new_idx = idx;
            egui::ComboBox::from_id_salt(entry.key)
                .selected_text(options[idx])
                .show_ui(ui, |ui| {
                    for (i, opt) in options.iter().enumerate() {
                        ui.selectable_value(&mut new_idx, i, *opt);
                    }
                });
            if new_idx != idx {
                **v = new_idx as u32;
                changed = true;
            }
        }
        (SettingValue::I32(v), SettingKind::Number { min, max, .. }) => {
            changed = ui
                .add(egui::DragValue::new(&mut **v).range(*min..=*max))
                .changed();
        }
        (SettingValue::I32(v), SettingKind::Slider { min, max, unit, .. }) => {
            let mut as_f = **v as f64;
            changed = ui
                .add(egui::Slider::new(&mut as_f, *min..=*max).suffix(*unit))
                .changed();
            **v = as_f.round() as i32;
        }
        (SettingValue::F32(v), SettingKind::Number { min, max, step }) => {
            changed = ui
                .add(
                    egui::DragValue::new(&mut **v)
                        .range(*min..=*max)
                        .speed(*step),
                )
                .changed();
        }
        (SettingValue::F32(v), SettingKind::Slider { min, max, unit, .. }) => {
            changed = ui
                .add(egui::Slider::new(&mut **v, (*min as f32)..=(*max as f32)).suffix(*unit))
                .changed();
        }
        (SettingValue::Text(v), SettingKind::Text) => {
            changed = ui
                .add(egui::TextEdit::singleline(&mut **v).desired_width(220.0))
                .changed();
        }
        (SettingValue::Text(v), SettingKind::Directory) => {
            ui.label((**v).as_str());
            if ui.small_button("…").clicked() {
                picks.push(PickRequest::Directory {
                    key: entry.key.to_string(),
                });
            }
        }
        (SettingValue::Text(v), SettingKind::File { .. }) => {
            ui.label((**v).as_str());
            if ui.small_button("…").clicked() {
                picks.push(PickRequest::File {
                    key: entry.key.to_string(),
                });
            }
        }
        (SettingValue::Choice(v), SettingKind::Dropdown { options }) => {
            let idx = (**v).min(options.len().saturating_sub(1));
            let mut new_idx = idx;
            egui::ComboBox::from_id_salt(entry.key)
                .selected_text(options[idx])
                .show_ui(ui, |ui| {
                    for (i, opt) in options.iter().enumerate() {
                        ui.selectable_value(&mut new_idx, i, *opt);
                    }
                });
            if new_idx != idx {
                **v = new_idx;
                changed = true;
            }
        }
        (value, kind) => {
            ui.label(format!("{} ({:?})", value.display(), kind));
        }
    }
    changed
}

/// Reset a single entry to its default representation. Only value shapes with
/// well-defined defaults are resettable here; the rest are covered by
/// category resets.
fn reset_one(entry: &mut SettingEntry<'_>, default: &str) {
    match &mut entry.value {
        SettingValue::Bool(v) => **v = default == "true",
        SettingValue::U32(v) => **v = default.parse().unwrap_or(0),
        SettingValue::I32(v) => **v = default.parse().unwrap_or(0),
        SettingValue::F32(v) => **v = default.parse().unwrap_or(0.0),
        SettingValue::Text(v) => v.clear(),
        SettingValue::Choice(v) => **v = 0,
    }
}

/// The keyboard shortcut editor: per-action rows with rebind capture,
/// conflict detection, reset and profile import/export.
fn keyboard_editor(
    ui: &mut Ui,
    tr: &Tr,
    bindings_json: &mut String,
    capture: &mut Option<noir_settings::schema::input::ActionBinding>,
    conflicts: &mut String,
    import_keys: &mut bool,
    export_keys: &mut bool,
) {
    use noir_settings::schema::input::{ActionBinding, ShortcutRegistry};

    ui.heading(tr.t("set.keys.title"));
    ui.separator();

    let mut registry: ShortcutRegistry = serde_json::from_str::<Vec<KeyBinding>>(bindings_json)
        .map(ShortcutRegistry::new)
        .unwrap_or_default();

    let conflict_list = registry.conflicts();
    if !conflict_list.is_empty() {
        let text = conflict_list
            .iter()
            .map(|(combo, actions)| {
                let names = actions
                    .iter()
                    .map(|a| a.default_label())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}: {}", combo, names)
            })
            .collect::<Vec<_>>()
            .join("; ");
        *conflicts = format!("{}: {text}", tr.t("set.keys.conflict"));
    }

    if capture.is_some() {
        ui.label(RichText::new(tr.t("set.keys.press_key")).strong());
    }

    let mut remove_binding: Option<ActionBinding> = None;
    ScrollArea::vertical().show(ui, |ui| {
        for action in ActionBinding::ALL {
            ui.horizontal(|ui| {
                ui.label(action.default_label());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let current = registry
                        .bindings()
                        .iter()
                        .find(|b| b.action == action)
                        .map(|b| b.combo.to_string())
                        .unwrap_or_else(|| "—".into());
                    if Some(action) == *capture {
                        ui.label(RichText::new("…").strong());
                    } else if ui.small_button(tr.t("set.keys.change")).clicked() {
                        *capture = Some(action);
                    }
                    if ui.small_button(tr.t("set.keys.unassign")).clicked() {
                        remove_binding = Some(action);
                    }
                    ui.monospace(current);
                });
            });
        }
    });

    if let Some(action) = remove_binding.take() {
        registry.unassign(action);
        *capture = None;
        persist_registry(&mut *bindings_json, &registry);
    }

    // Capture state is handled by the app (it sees raw key events).
    ui.horizontal(|ui| {
        if ui.button(tr.t("set.keys.reset")).clicked() {
            registry.reset_to_defaults();
            persist_registry(&mut *bindings_json, &registry);
        }
        if ui.button(tr.t("set.keys.import")).clicked() {
            *import_keys = true;
        }
        if ui.button(tr.t("set.keys.export")).clicked() {
            *export_keys = true;
        }
    });
}

fn persist_registry(
    bindings_json: &mut String,
    registry: &noir_settings::schema::input::ShortcutRegistry,
) {
    if let Ok(json) = serde_json::to_string(registry.bindings()) {
        *bindings_json = json;
    }
}

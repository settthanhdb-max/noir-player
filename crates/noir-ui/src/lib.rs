//! NOIR Player desktop UI (egui/eframe).
//!
//! The UI never touches decoder internals: it sends [`EngineCommand`]s to the
//! [`PlaybackController`] and consumes [`PlaybackEvent`]s, regardless of the
//! backend behind the trait.

pub mod app;
pub mod dialogs;
pub mod i18n;
pub mod icons;
pub mod player;
pub mod playlist_panel;
pub mod screens;
pub mod settings_ui;
pub mod shortcuts;
pub mod state;
pub mod theme;

pub use app::NoirApp;

/// Launch the native window. Returns when the application exits.
/// Launch the native window. `initial_files` are media paths passed on the
/// command line (file-association opens); they are queued and playback
/// starts with the first one. Returns when the application exits.
pub fn run(initial_files: Vec<String>) -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("NOIR Player")
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([720.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native(
        "NOIR Player",
        options,
        Box::new(move |cc| Ok(Box::new(NoirApp::new_with_files(cc, &initial_files)))),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
}

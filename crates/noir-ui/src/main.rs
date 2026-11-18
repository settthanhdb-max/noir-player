//! NOIR Player binary entry point.
//!
//! In release builds on Windows the console window is suppressed; logs go to
//! the log file instead. Debug builds keep the console for live output.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    let paths = noir_platform::PlatformPaths::resolve();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("warning: could not create data directories: {e}");
    }

    #[cfg(all(windows, not(debug_assertions)))]
    {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths.log_file())?;
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false)
            .init();
    }

    #[cfg(not(all(windows, not(debug_assertions))))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_writer(std::io::stderr)
            .init();
    }

    tracing::info!(log_file = %paths.log_file().display(), "starting NOIR Player");

    let files: Vec<String> = std::env::args().skip(1).collect();

    // Single instance on Windows release: a double-clicked media file while
    // the player is already running is handed off to the running instance.
    #[cfg(all(windows, not(debug_assertions)))]
    let _guard = match noir_platform::single_instance::acquire("Local\\noir-player-singleton") {
        Ok(g) => Some(g),
        Err(()) => {
            if !files.is_empty() {
                match noir_platform::single_instance::signal_open_request(&paths.data_dir, &files) {
                    Ok(()) => tracing::info!("handed {files:?} to the running instance"),
                    Err(e) => tracing::error!("failed to hand off files: {e}"),
                }
            }
            return Ok(());
        }
    };
    #[cfg(not(all(windows, not(debug_assertions))))]
    let _guard = None::<noir_platform::single_instance::InstanceGuard>;

    noir_ui::run(files)
}

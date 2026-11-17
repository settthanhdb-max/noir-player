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

    noir_ui::run()
}

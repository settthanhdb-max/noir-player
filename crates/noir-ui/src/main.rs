//! NOIR Player binary entry point.

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let paths = noir_platform::PlatformPaths::resolve();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("warning: could not create data directories: {e}");
    }
    tracing::info!(log_file = %paths.log_file().display(), "starting NOIR Player");

    noir_ui::run()
}

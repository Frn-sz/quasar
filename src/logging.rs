use {
    chrono::Local,
    std::{
        fs::{self, OpenOptions},
        path::PathBuf,
    },
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
};

pub fn init_logging(debug: bool) -> std::io::Result<()> {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir)?;

    let timestamp = Local::now().format("%Y-%m-%d");
    let log_file = log_dir.join(format!("{}.log", timestamp));

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let filter_level = if debug { "debug" } else { "info" };
    let env_filter = EnvFilter::new(filter_level);

    let console_layer = fmt::layer().with_target(false).with_ansi(true).compact();

    let file_layer = fmt::layer()
        .with_writer(file)
        .with_target(false)
        .with_ansi(false)
        .compact();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging to: {}", log_file.display());

    Ok(())
}

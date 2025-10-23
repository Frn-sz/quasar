use {
    clap::Parser,
    quasar::{Quasar, config::QuasarServerConfig},
    tracing::error,
};

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = match QuasarServerConfig::from_file(&cli.config) {
        Ok(config) => config,
        Err(e) => {
            error!("Error: failed to load server config file: {e}");
            return;
        }
    };

    let _profiler = dhat::Profiler::new_heap();

    let mut app = Quasar::new(config);

    if let Err(e) = app.run().await {
        error!("Quasar failed to run: {}", e);
    }
}

use clap::Parser;
use quasar::{Quasar, config::QuasarConfig};
use tracing::error;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = get_config(&cli).expect("Failed to load config");

    let mut app = Quasar::new(config);

    if let Err(e) = app.run().await {
        error!("Quasar failed to run: {}", e);
    }
}

fn get_config(cli: &Cli) -> Result<QuasarConfig, String> {
    match QuasarConfig::from_file(&cli.config) {
        Ok(config) => Ok(config),
        Err(e) => {
            error!("{e}");
            Err(format!("Error loading config file: {e}"))
        }
    }
}

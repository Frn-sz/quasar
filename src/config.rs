use config::{Config, ConfigError, File, FileFormat};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct QuasarServerConfig {
    pub grpc: GrpcConfig,
    pub http: HttpConfig,
    pub metrics: MetricsConfig,
    pub debug: bool,
}

impl QuasarServerConfig {
    pub fn from_file(config_path: &str) -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::new(config_path, FileFormat::Toml));

        let config: QuasarServerConfig = builder.build()?.try_deserialize()?;

        Ok(config)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct QuasarClientConfig {
    pub grpc: GrpcConfig,
    pub http: HttpConfig,
    pub debug: bool,
    pub tasks: usize,
    // Chance (0-100) of creating a new account instead of making a transfer
    pub create_chance: u8,
    pub deposit_chance: u8,
}

impl QuasarClientConfig {
    pub fn from_file(config_path: &str) -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::new(config_path, FileFormat::Toml));

        let config: QuasarClientConfig = builder.build()?.try_deserialize()?;

        Ok(config)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct GrpcConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct HttpConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct MetricsConfig {
    pub remote_write_url: String,
    pub push_interval_seconds: u64,
}

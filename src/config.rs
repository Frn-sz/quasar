use config::{Config, ConfigError, File, FileFormat};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct QuasarConfig {
    pub grpc: GrpcConfig,
    pub http: HttpConfig,
    pub debug: bool,
}

impl QuasarConfig {
    pub fn from_file(config_path: &str) -> Result<Self, ConfigError> {
        let builder = Config::builder().add_source(File::new(config_path, FileFormat::Toml));

        let config: QuasarConfig = builder.build()?.try_deserialize()?;

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

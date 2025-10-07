#[derive(Clone, Debug)]
pub struct QuasarConfig {
    pub grpc: GrpcConfig,
    pub http: HttpConfig,
}

#[derive(Clone, Debug)]
pub struct GrpcConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub address: String,
    pub port: u16,
}

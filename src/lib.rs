use std::sync::{Arc, RwLock};

pub mod config;
pub mod ledger;
pub mod models;
pub mod transaction_processor;

pub struct Quasar {
    // TransactionProcessor has internal locking, so we can use it directly.
    pub transaction_processor: Arc<RwLock<transaction_processor::TransactionProcessor>>,
    pub config: config::QuasarConfig,
}

impl Quasar {
    pub fn new(config: config::QuasarConfig) -> Self {
        let ledger = Arc::new(RwLock::new(ledger::Ledger::new()));

        // Cheap clone of Arc
        let transaction_processor = Arc::new(RwLock::new(
            transaction_processor::TransactionProcessor::new(ledger.clone()),
        ));

        Quasar {
            transaction_processor,
            config,
        }
    }

    pub async fn run(&mut self) {
        let grpc_processor = self.transaction_processor.clone();
        let grpc_config = self.config.grpc.clone();

        let http_handle = tokio::spawn(async move { todo!() });
        let grpc_handle = tokio::spawn(async move { todo!() });

        if let Err(e) = tokio::try_join!(http_handle, grpc_handle) {
            eprintln!("Error running servers: {:?}", e);
        }
    }
}

use std::sync::{Arc, RwLock};

use tokio::signal::ctrl_c;
use tracing::error;

use crate::{grpc_service::start_grpc_service, logging::init_logging};

pub mod config;
pub mod grpc_service;
pub mod ledger;
pub mod logging;
pub mod models;
pub mod transaction_processor;

pub struct Quasar {
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

    pub async fn run(&mut self) -> Result<(), String> {
        let (shutdown_sender, _) = tokio::sync::broadcast::channel::<()>(1);
        let mut services = tokio::task::JoinSet::new();
        let _logging_guard = init_logging(self.config.debug);

        // TODO
        // let http_handle = tokio::spawn(async move { todo!() });

        {
            let grpc_processor = Arc::clone(&self.transaction_processor);
            let grpc_config = self.config.grpc.clone();
            let shutdown_receiver = shutdown_sender.subscribe();
            services.spawn(async move {
                start_grpc_service(grpc_config, grpc_processor, shutdown_receiver).await
            })
        };

        tokio::select! {
            _ = ctrl_c() => {
                shutdown_sender.send(()).map_err(|e| e.to_string())?;
                services.abort_all();
                tracing::info!("Shutdown signal received, stopping services...");
            }
            Some(res) = services.join_next() => {
                error!("Error in task: {:?}", res);
            }
        }

        Ok(())
    }
}

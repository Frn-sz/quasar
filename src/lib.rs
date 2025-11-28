use {
    crate::{
        grpc_server::start_grpc_service, grpc_server::start_grpc_service, ledger::Ledger,
        logging::init_logging, logging::init_logging, metrics::handler::start_metrics_pusher,
        persistence::Persistence, transaction_processor::TransactionProcessor,
    },
    std::sync::{Arc, RwLock},
    tokio::signal::ctrl_c,
    tracing::{error, info},
};

pub mod config;
pub mod grpc_server;
pub mod ledger;
pub mod logging;
#[macro_use]
pub mod macros;
pub mod metrics;
pub mod models;
pub mod persistence;
pub mod transaction_processor;

pub struct Quasar {
    pub transaction_processor: Arc<RwLock<TransactionProcessor>>,
    pub config: config::QuasarServerConfig,
    pub persistence: Persistence,
    ledger: Arc<RwLock<Ledger>>,
}

impl Quasar {
    pub fn new(config: config::QuasarServerConfig) -> Self {
        let persistence = Persistence::new(&config.persistence.db_path)
            .expect("Failed to initialize persistence");
        let accounts = persistence
            .load_accounts()
            .expect("Failed to load accounts");
        let ledger = Arc::new(RwLock::new(Ledger::new(accounts)));

        // Cheap clone of Arc
        let transaction_processor =
            Arc::new(RwLock::new(TransactionProcessor::new(ledger.clone())));

        Quasar {
            transaction_processor,
            config,
            persistence,
            ledger,
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        let (shutdown_sender, _) = tokio::sync::broadcast::channel::<()>(1);
        let mut services = tokio::task::JoinSet::new();
        let _logging_guard = init_logging(self.config.debug);

        // Metrics pusher service
        let metrics_config = self.config.metrics.clone();
        let shutdown_receiver = shutdown_sender.subscribe();

        // TODO: add REST API service here
        {
            info!(
                "Initializing with {} accounts",
                self.ledger.read().unwrap().accounts.read().unwrap().len()
            );

            services.spawn(async move {
                start_metrics_pusher(metrics_config, shutdown_receiver).await;
            });
        }

        // gRPC service
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

                let accounts = self.ledger.read().unwrap().accounts.read().unwrap().clone();
                self.persistence.save_accounts(&accounts).expect("Failed to save accounts");

                tracing::info!("Accounts saved successfully");
            }
            Some(res) = services.join_next() => {
                error!("Error in task: {:?}", res);
            }
        }

        Ok(())
    }
}

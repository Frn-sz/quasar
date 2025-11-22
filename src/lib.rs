use {
    crate::{
        grpc_server::start_grpc_service, ledger::Ledger, logging::init_logging,
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
pub mod metrics;
pub mod models;
pub mod persistence;
pub mod transaction_processor;

pub struct Quasar {
    pub transaction_processor: Arc<RwLock<TransactionProcessor>>,
    pub config: config::QuasarServerConfig,
    pub persistence: Persistence,
    ledger: Arc<Ledger>,
}

impl Quasar {
    pub fn new(config: config::QuasarServerConfig) -> Self {
        let persistence = Persistence::new(&config.persistence.db_path)
            .expect("Failed to initialize persistence");

        let accounts = persistence
            .load_accounts()
            .expect("Failed to load accounts");

        let ledger = Arc::new(Ledger::new(accounts));

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

        info!("Initializing with {} accounts", self.ledger.accounts.len());

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

                self.persistence.save_accounts(&self.ledger.accounts).expect("Failed to save accounts");

                tracing::info!("Accounts saved successfully");
            }
            Some(res) = services.join_next() => {
                error!("Error in task: {:?}", res);
            }
        }

        Ok(())
    }
}

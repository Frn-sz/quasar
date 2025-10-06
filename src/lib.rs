use std::sync::{Arc, RwLock};

pub mod ledger;
pub mod models;
pub mod transaction_processor;

pub struct Quasar {
    // Ledger has internal locking, so we can use it directly.
    pub ledger: Arc<RwLock<ledger::Ledger>>,
    // Same for TransactionProcessor.
    pub transaction_processor: transaction_processor::TransactionProcessor,
}

impl Quasar {
    pub fn new() -> Self {
        let ledger = Arc::new(RwLock::new(ledger::Ledger::new()));

        // Cheap clone of Arc
        let transaction_processor =
            transaction_processor::TransactionProcessor::new(ledger.clone());

        Quasar {
            ledger,
            transaction_processor,
        }
    }

    pub async fn run(&mut self) {}
}

//! Transaction Processor module for processing and validating transactions.
//! This module interacts with the Ledger module to commit valid transactions.

use std::sync::{Arc, RwLock};

use crate::{
    ledger::interface::LedgerInterface,
    transaction_processor::interface::TransactionProcessorInterface,
};

pub mod error;
pub mod interface;

pub struct TransactionProcessor {
    // This must be improved later to avoid locking the entire ledger for each transaction.
    ledger: Arc<RwLock<dyn LedgerInterface>>,
}

impl TransactionProcessorInterface for TransactionProcessor {
    fn process_transaction(
        &mut self,
        transaction: crate::models::Transaction,
    ) -> Result<(), error::TransactionProcessorError> {
        Ok(())
    }

    fn validate_transaction(
        &self,
        transaction: &crate::models::Transaction,
    ) -> Result<(), error::TransactionProcessorError> {
        todo!()
    }

    fn simulate_transaction(
        &self,
        transaction: &crate::models::Transaction,
    ) -> Result<(), error::TransactionProcessorError> {
        todo!()
    }
}

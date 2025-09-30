//! Transaction Processor module for processing and validating transactions.
//! This module interacts with the Ledger module to commit valid transactions.

pub mod error;
pub mod interface;

use {
    crate::{
        ledger::interface::LedgerInterface,
        transaction_processor::{
            error::TransactionProcessorError, interface::TransactionProcessorInterface,
        },
    },
    std::sync::{Arc, RwLock},
};

pub struct TransactionProcessor {
    // This must be improved later to avoid locking the entire ledger for each transaction.
    ledger: Arc<RwLock<dyn LedgerInterface>>,
}

impl TransactionProcessor {
    pub fn new(ledger: Arc<RwLock<dyn LedgerInterface + Send + Sync>>) -> Self {
        TransactionProcessor { ledger }
    }
}

impl TransactionProcessorInterface for TransactionProcessor {
    fn process_transaction(
        &mut self,
        transaction: crate::models::Transaction,
    ) -> Result<(), TransactionProcessorError> {
        let mut ledger = self.ledger.write().unwrap();

        if ledger.is_transaction_processed(transaction.id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        let mut source_account = ledger.get_account(transaction.source_account_id)?;
        let mut dest_account = ledger.get_account(transaction.destination_account_id)?;

        if source_account.balance < transaction.amount {
            return Err(TransactionProcessorError::InsufficientFunds);
        }

        source_account.balance = source_account.balance.saturating_sub(transaction.amount);
        dest_account.balance = dest_account.balance.saturating_add(transaction.amount);

        ledger.commit_updates(&transaction, &mut source_account, &mut dest_account)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::{Ledger, error::LedgerError};
    use crate::models::{Transaction, TransactionStatus};
    use crate::transaction_processor::error::TransactionProcessorError;
    use chrono::Utc;
    use std::sync::{Arc, RwLock};
    use uuid::Uuid;

    fn setup() -> (TransactionProcessor, Arc<RwLock<Ledger>>, Uuid, Uuid) {
        let ledger = Arc::new(RwLock::new(Ledger::new()));
        let processor = TransactionProcessor::new(ledger.clone());

        let mut ledger_lock = ledger.write().unwrap();
        let source_id = ledger_lock.create_account().unwrap();
        let dest_id = ledger_lock.create_account().unwrap();

        let mut source_account = ledger_lock.get_account(source_id).unwrap();
        let mut dest_account = ledger_lock.get_account(dest_id).unwrap();
        source_account.balance = 1000;

        let tx = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 0,
            timestamp: Utc::now(),
            status: TransactionStatus::Completed,
        };

        ledger_lock
            .commit_updates(&tx, &mut source_account, &mut dest_account)
            .unwrap();

        drop(ledger_lock);

        (processor, ledger, source_id, dest_id)
    }

    #[test]
    fn test_process_successful_transaction() {
        let (mut processor, ledger, source_id, dest_id) = setup();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 100,
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        };

        let result = processor.process_transaction(transaction);
        assert!(result.is_ok());

        let ledger_lock = ledger.read().unwrap();
        let source_account = ledger_lock.get_account(source_id).unwrap();
        let dest_account = ledger_lock.get_account(dest_id).unwrap();

        assert_eq!(source_account.balance, 900);
        assert_eq!(dest_account.balance, 100);
    }

    #[test]
    fn test_process_transaction_insufficient_funds() {
        let (mut processor, _, source_id, dest_id) = setup();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 2000, // More than the balance
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        };

        let result = processor.process_transaction(transaction);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TransactionProcessorError::InsufficientFunds
        ));
    }

    #[test]
    fn test_process_transaction_account_not_found() {
        let (mut processor, _, source_id, _) = setup();
        let non_existing_id = Uuid::new_v4();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: non_existing_id,
            amount: 100,
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        };

        let result = processor.process_transaction(transaction);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TransactionProcessorError::Ledger(LedgerError::AccountNotFound)
        ));
    }

    #[test]
    fn test_process_idempotency_transaction_already_processed() {
        let (mut processor, _, source_id, dest_id) = setup();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 100,
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        };

        let result1 = processor.process_transaction(transaction.clone());
        assert!(result1.is_ok());

        let result2 = processor.process_transaction(transaction);
        assert!(result2.is_err());
        assert!(matches!(
            result2.err().unwrap(),
            TransactionProcessorError::TransactionAlreadyProcessed
        ));
    }
}

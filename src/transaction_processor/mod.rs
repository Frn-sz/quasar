//! Transaction Processor module for processing and validating transactions.
//! This module interacts with the Ledger module to commit valid transactions.

pub mod error;
pub mod interface;

use {
    crate::{
        ledger::interface::LedgerInterface,
        models::{CreateAccountInstruction, InstructionType, Transaction, TransferInstruction},
        transaction_processor::{
            error::TransactionProcessorError, interface::TransactionProcessorInterface,
        },
    },
    std::sync::{Arc, RwLock},
    uuid::Uuid,
};

pub struct TransactionProcessor {
    ledger: Arc<RwLock<dyn LedgerInterface + Send + Sync>>,
}

impl TransactionProcessor {
    pub fn new(ledger: Arc<RwLock<dyn LedgerInterface + Send + Sync>>) -> Self {
        TransactionProcessor { ledger }
    }

    fn process_transfer(
        &mut self,
        transaction_id: Uuid,
        instruction: TransferInstruction,
    ) -> Result<(), TransactionProcessorError> {
        let mut ledger = self.ledger.write().unwrap();

        if ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        let mut source_account = ledger.get_account(instruction.source_account_id)?;
        let mut dest_account = ledger.get_account(instruction.destination_account_id)?;

        if source_account.balance < instruction.amount {
            return Err(TransactionProcessorError::InsufficientFunds);
        }

        source_account.balance = source_account.balance.saturating_sub(instruction.amount);
        dest_account.balance = dest_account.balance.saturating_add(instruction.amount);

        ledger.commit_transfer(
            transaction_id,
            &instruction,
            &mut source_account,
            &mut dest_account,
        )?;

        Ok(())
    }

    fn process_create_account(
        &mut self,
        transaction_id: Uuid,
        instruction: CreateAccountInstruction,
    ) -> Result<(), TransactionProcessorError> {
        let mut ledger = self.ledger.write().unwrap();

        if ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        ledger.create_account(instruction.keys)?;
        ledger.mark_transaction_processed(transaction_id)?;

        Ok(())
    }
}

impl TransactionProcessorInterface for TransactionProcessor {
    fn process_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), TransactionProcessorError> {
        match transaction.instruction_type {
            InstructionType::Transfer(inst) => self.process_transfer(transaction.id, inst),
            InstructionType::CreateAccount(inst) => {
                self.process_create_account(transaction.id, inst)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::ledger::Ledger;
    use crate::models::{CreateAccountInstruction, Key, TransactionStatus};

    // Helper to set up test environment with existing accounts
    fn setup_for_transfer() -> (TransactionProcessor, Arc<RwLock<Ledger>>, Uuid, Uuid) {
        let ledger = Arc::new(RwLock::new(Ledger::new()));
        let processor = TransactionProcessor::new(ledger.clone());

        let mut ledger_lock = ledger.write().unwrap();
        let source_id = ledger_lock.create_account(vec![]).unwrap();
        let dest_id = ledger_lock.create_account(vec![]).unwrap();

        let mut source_account = ledger_lock.get_account(source_id).unwrap();
        source_account.balance = 1000;
        let mut dest_account = ledger_lock.get_account(dest_id).unwrap();

        let transfer_inst = TransferInstruction {
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 0,
        };

        // Commit inicial para acertar o saldo
        ledger_lock
            .commit_transfer(
                Uuid::new_v4(),
                &transfer_inst,
                &mut source_account,
                &mut dest_account,
            )
            .unwrap();

        drop(ledger_lock);

        (processor, ledger, source_id, dest_id)
    }

    #[test]
    fn test_process_create_account_transaction() {
        let ledger = Arc::new(RwLock::new(Ledger::new()));
        let mut processor = TransactionProcessor::new(ledger.clone());

        let transaction = Transaction {
            id: Uuid::new_v4(),
            instruction_type: InstructionType::CreateAccount(CreateAccountInstruction {
                keys: vec![Key::Email("test@test.com".to_string())],
                id: Uuid::new_v4(),
            }),
            timestamp: Utc::now(),
            status: TransactionStatus::Pending,
        };

        let result = processor.process_transaction(transaction);
        assert!(result.is_ok());

        let ledger_lock = ledger.read().unwrap();
        assert_eq!(ledger_lock.accounts.read().unwrap().len(), 1);
    }

    #[test]
    fn test_process_successful_transfer() {
        let (mut processor, ledger, source_id, dest_id) = setup_for_transfer();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            instruction_type: InstructionType::Transfer(TransferInstruction {
                source_account_id: source_id,
                destination_account_id: dest_id,
                amount: 100,
            }),
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
    fn test_process_transfer_insufficient_funds() {
        let (mut processor, _, source_id, dest_id) = setup_for_transfer();

        let transaction = Transaction {
            id: Uuid::new_v4(),
            instruction_type: InstructionType::Transfer(TransferInstruction {
                source_account_id: source_id,
                destination_account_id: dest_id,
                amount: 2000, // More than available balance
            }),
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
}

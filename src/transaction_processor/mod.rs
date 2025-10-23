//! Transaction Processor module for processing and validating transactions.
//! This module interacts with the Ledger module to commit valid transactions.

pub mod error;
pub mod interface;

use {
    crate::{
        ledger::interface::LedgerInterface,
        metrics::{
            ACCOUNT_CREATION_TIME_SECONDS, DEPOSIT_TIME_SECONDS, GET_BALANCE_TIME_SECONDS,
            TRANSACTION_PROCESSING_TIME_SECONDS, TRANSACTIONS_PROCESSED_TOTAL,
            TRANSFER_TIME_SECONDS,
        },
        models::{
            CreateAccountInstruction, DepositInstruction, Instruction, Transaction,
            TransferInstruction,
        },
        transaction_processor::{
            error::TransactionProcessorError,
            interface::{TransactionProcessorInterface, TransactionResult},
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
    ) -> Result<TransactionResult, TransactionProcessorError> {
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

        Ok(TransactionResult::Success)
    }

    fn process_create_account(
        &mut self,
        transaction_id: Uuid,
        instruction: CreateAccountInstruction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        let mut ledger = self.ledger.write().unwrap();

        if ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        let created_account_id = ledger.create_account(instruction.keys)?;

        ledger.mark_transaction_processed(transaction_id)?;

        Ok(TransactionResult::AccountCreated(created_account_id))
    }

    fn process_deposit(
        &mut self,
        transaction_id: Uuid,
        instruction: DepositInstruction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        let mut ledger = self.ledger.write().unwrap();

        if ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        ledger.deposit_into_account(instruction.destination_account_id, instruction.amount)?;

        ledger.mark_transaction_processed(transaction_id)?;

        Ok(TransactionResult::Success)
    }

    fn get_balance(
        &self,
        account_id: Uuid,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        let ledger = self
            .ledger
            .read()
            .map_err(|_| TransactionProcessorError::FailedToAcquireLedgerLock)?;

        let account = ledger.get_account(account_id)?;

        Ok(TransactionResult::Balance(account.balance))
    }
}

impl TransactionProcessorInterface for TransactionProcessor {
    fn process_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        TRANSACTIONS_PROCESSED_TOTAL.inc();
        measure!(TRANSACTION_PROCESSING_TIME_SECONDS, {
            match transaction.instruction {
                Instruction::Transfer(inst) => measure!(TRANSFER_TIME_SECONDS, {
                    self.process_transfer(transaction.id, inst)
                }),
                Instruction::CreateAccount(inst) => {
                    measure!(ACCOUNT_CREATION_TIME_SECONDS, {
                        self.process_create_account(transaction.id, inst)
                    })
                }
                Instruction::Deposit(deposit_instruction) => {
                    measure!(DEPOSIT_TIME_SECONDS, {
                        self.process_deposit(transaction.id, deposit_instruction)
                    })
                }
                Instruction::GetBalance(get_balance_instruction) => {
                    measure!(GET_BALANCE_TIME_SECONDS, {
                        self.get_balance(get_balance_instruction.account_id)
                    })
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            ledger::Ledger,
            models::{CreateAccountInstruction, Key, TransactionStatus},
        },
        chrono::Utc,
    };

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

        // Initial commit to set the balance
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
            instruction: Instruction::CreateAccount(CreateAccountInstruction {
                keys: vec![Key::Email("test@test.com".to_string())],
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
            instruction: Instruction::Transfer(TransferInstruction {
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
            instruction: Instruction::Transfer(TransferInstruction {
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

//! Transaction Processor module for processing and validating transactions.
//! This module interacts with the Ledger module to commit valid transactions.

pub mod error;
pub mod interface;

use {
    crate::{
        ledger::interface::LedgerInterface,
        models::{
            CreateAccountInstruction, DepositInstruction, Instruction, Transaction,
            TransferInstruction,
        },
        transaction_processor::{
            error::TransactionProcessorError,
            interface::{TransactionProcessorInterface, TransactionResult},
        },
    },
    dashmap::DashMap,
    std::sync::Arc,
    uuid::Uuid,
};

pub struct TransactionProcessor {
    pub ledger: Arc<dyn LedgerInterface + Send + Sync>,
    pub transactions: DashMap<Uuid, Transaction>,
}

impl TransactionProcessor {
    pub fn new(
        ledger: Arc<dyn LedgerInterface + Send + Sync>,
        transactions: DashMap<Uuid, Transaction>,
    ) -> Self {
        TransactionProcessor {
            ledger,
            transactions,
        }
    }

    fn process_transfer(
        &self,
        transaction_id: Uuid,
        instruction: TransferInstruction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        if self.ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        let mut source_account = self.ledger.get_account(instruction.source_account_id)?;
        let mut dest_account = self
            .ledger
            .get_account(instruction.destination_account_id)?;

        if source_account.balance < instruction.amount {
            return Err(TransactionProcessorError::InsufficientFunds);
        }

        source_account.balance = source_account.balance.saturating_sub(instruction.amount);
        dest_account.balance = dest_account.balance.saturating_add(instruction.amount);

        self.ledger.commit_transfer(
            transaction_id,
            &instruction,
            &mut source_account,
            &mut dest_account,
        )?;

        Ok(TransactionResult::Success)
    }

    fn process_create_account(
        &self,
        transaction_id: Uuid,
        instruction: CreateAccountInstruction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        if self.ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        let created_account_id = self.ledger.create_account(instruction.keys)?;
        self.ledger.mark_transaction_processed(transaction_id)?;

        Ok(TransactionResult::AccountCreated(created_account_id))
    }

    fn process_deposit(
        &self,
        transaction_id: Uuid,
        instruction: DepositInstruction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        if self.ledger.is_transaction_processed(transaction_id)? {
            return Err(TransactionProcessorError::TransactionAlreadyProcessed);
        }

        self.ledger
            .deposit_into_account(instruction.destination_account_id, instruction.amount)?;
        self.ledger.mark_transaction_processed(transaction_id)?;

        Ok(TransactionResult::Success)
    }

    fn get_balance(
        &self,
        account_id: Uuid,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        let account = self.ledger.get_account(account_id)?;

        Ok(TransactionResult::Balance(account.balance))
    }
}

impl TransactionProcessorInterface for TransactionProcessor {
    fn process_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionResult, TransactionProcessorError> {
        self.transactions
            .insert(transaction.id, transaction.clone());

        match transaction.instruction {
            Instruction::Transfer(inst) => self.process_transfer(transaction.id, inst),
            Instruction::CreateAccount(inst) => self.process_create_account(transaction.id, inst),
            Instruction::Deposit(deposit_instruction) => {
                self.process_deposit(transaction.id, deposit_instruction)
            }
            Instruction::GetBalance(get_balance_instruction) => {
                self.get_balance(get_balance_instruction.account_id)
            }
        }
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
        dashmap::{DashMap, DashSet},
    };

    // Helper to set up test environment with existing accounts
    fn setup_for_transfer() -> (
        TransactionProcessor,
        Arc<dyn LedgerInterface + Send + Sync>,
        Uuid,
        Uuid,
    ) {
        let ledger = Arc::new(Ledger::new(DashMap::new(), DashSet::new()));
        let processor = TransactionProcessor::new(ledger.clone(), DashMap::new());

        let source_id = ledger.create_account(vec![]).unwrap();
        let dest_id = ledger.create_account(vec![]).unwrap();

        let mut source_account = ledger.get_account(source_id).unwrap();
        source_account.balance = 1000;
        let mut dest_account = ledger.get_account(dest_id).unwrap();

        let transfer_inst = TransferInstruction {
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 0,
        };

        // Initial commit to set the balance
        ledger
            .commit_transfer(
                Uuid::new_v4(),
                &transfer_inst,
                &mut source_account,
                &mut dest_account,
            )
            .unwrap();

        (processor, ledger, source_id, dest_id)
    }

    #[test]
    fn test_process_create_account_transaction() {
        let ledger = Arc::new(Ledger::new(DashMap::new(), DashSet::new()));
        let processor = TransactionProcessor::new(ledger.clone(), DashMap::new());

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

        assert_eq!(ledger.accounts.len(), 1);
    }

    #[test]
    fn test_process_successful_transfer() {
        let (processor, ledger, source_id, dest_id) = setup_for_transfer();

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

        let source_account = ledger.get_account(source_id).unwrap();
        let dest_account = ledger.get_account(dest_id).unwrap();

        assert_eq!(source_account.balance, 900);
        assert_eq!(dest_account.balance, 100);
    }

    #[test]
    fn test_process_transfer_insufficient_funds() {
        let (processor, _, source_id, dest_id) = setup_for_transfer();

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

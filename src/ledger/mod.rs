pub mod error;
pub mod interface;
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    ledger::{error::LedgerError, interface::LedgerInterface},
    models::{Account, HistoricTransfer, Key, TransferInstruction},
};

pub struct Ledger {
    // For now using a global lock for baseline implementation; must be optimized later.
    pub accounts: RwLock<HashMap<Uuid, Account>>,
    // To prevent processing the same transaction multiple times (ensure idempotency).
    processed_transactions: RwLock<HashSet<Uuid>>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: RwLock::new(HashMap::new()),
            processed_transactions: RwLock::new(HashSet::new()),
        }
    }

    fn acquire_accounts_write_lock(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, HashMap<Uuid, Account>>, LedgerError> {
        self.accounts
            .write()
            .map_err(|_| LedgerError::FailedToAcquireAccountsWriteLock)
    }

    fn acquire_accounts_read_lock(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, HashMap<Uuid, Account>>, LedgerError> {
        self.accounts
            .read()
            .map_err(|_| LedgerError::FailedToAcquireAccountsReadLock)
    }

    fn acquire_transactions_write_lock(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, HashSet<Uuid>>, LedgerError> {
        self.processed_transactions
            .write()
            .map_err(|_| LedgerError::FailedToAcquireTransactionsWriteLock)
    }

    fn acquire_transactions_read_lock(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, HashSet<Uuid>>, LedgerError> {
        self.processed_transactions
            .read()
            .map_err(|_| LedgerError::FailedToAcquireTransactionsReadLock)
    }
}

impl LedgerInterface for Ledger {
    fn create_account(&mut self, keys: Vec<Key>) -> Result<Uuid, LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;
        let (account_id, account) = Account::new(keys);
        accounts.insert(account_id, account);
        Ok(account_id)
    }

    fn get_account(&self, id: Uuid) -> Result<Account, LedgerError> {
        let accounts = self.acquire_accounts_read_lock()?;
        accounts
            .get(&id)
            .cloned()
            .ok_or(LedgerError::AccountNotFound)
    }

    fn commit_transfer(
        &mut self,
        transaction_id: Uuid,
        instruction: &TransferInstruction,
        source_account: &mut Account,
        dest_account: &mut Account,
    ) -> Result<(), LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;
        let mut processed_transactions = self.acquire_transactions_write_lock()?;

        // Safeguard check, although the processor should handle this.
        if processed_transactions.contains(&transaction_id) {
            return Err(LedgerError::TransactionAlreadyProcessed);
        }

        // Add instruction to history
        source_account.transaction_history.push(HistoricTransfer {
            transaction_id,
            instruction: instruction.clone(),
            timestamp: Utc::now(),
        });

        dest_account.transaction_history.push(HistoricTransfer {
            transaction_id,
            instruction: instruction.clone(),
            timestamp: Utc::now(),
        });

        accounts.insert(source_account.uuid, source_account.clone());
        accounts.insert(dest_account.uuid, dest_account.clone());

        processed_transactions.insert(transaction_id);

        Ok(())
    }

    fn is_transaction_processed(&self, transaction_id: Uuid) -> Result<bool, LedgerError> {
        let processed_transactions = self.acquire_transactions_read_lock()?;
        Ok(processed_transactions.contains(&transaction_id))
    }

    fn mark_transaction_processed(&mut self, transaction_id: Uuid) -> Result<(), LedgerError> {
        let mut processed_transactions = self.acquire_transactions_write_lock()?;
        processed_transactions.insert(transaction_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::models::{Key, TransferInstruction};

    #[test]
    fn test_create_account() {
        let mut ledger = Ledger::new();
        let keys = vec![Key::Email("test@test.com".to_string())];
        let account_id_result = ledger.create_account(keys);
        assert!(account_id_result.is_ok());
        let account_id = account_id_result.unwrap();

        let accounts_lock = ledger.accounts.read().unwrap();
        assert!(accounts_lock.contains_key(&account_id));
        assert_eq!(accounts_lock.get(&account_id).unwrap().keys.len(), 1);
    }

    #[test]
    fn test_get_existing_account() {
        let mut ledger = Ledger::new();
        let account_id = ledger.create_account(vec![]).unwrap();
        let account_result = ledger.get_account(account_id);
        assert!(account_result.is_ok());
        assert_eq!(account_result.unwrap().uuid, account_id);
    }

    #[test]
    fn test_commit_transfer_and_is_processed() {
        let mut ledger = Ledger::new();
        let source_id = ledger.create_account(vec![]).unwrap();
        let dest_id = ledger.create_account(vec![]).unwrap();

        let mut source_account = ledger.get_account(source_id).unwrap();
        let mut dest_account = ledger.get_account(dest_id).unwrap();
        source_account.balance = 50;
        dest_account.balance = 150;

        let transaction_id = Uuid::new_v4();
        let instruction = TransferInstruction {
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 50,
        };

        // Before commit
        assert!(!ledger.is_transaction_processed(transaction_id).unwrap());

        // Commit
        let commit_result = ledger.commit_transfer(
            transaction_id,
            &instruction,
            &mut source_account,
            &mut dest_account,
        );
        assert!(commit_result.is_ok());

        // After commit
        assert!(ledger.is_transaction_processed(transaction_id).unwrap());

        let final_source_account = ledger.get_account(source_id).unwrap();
        let final_dest_account = ledger.get_account(dest_id).unwrap();

        assert_eq!(final_source_account.balance, 50);
        assert_eq!(final_dest_account.balance, 150);
        assert_eq!(final_source_account.transaction_history.len(), 1);
        assert_eq!(final_dest_account.transaction_history.len(), 1);
    }

    #[test]
    fn test_mark_transaction_as_processed() {
        let mut ledger = Ledger::new();
        let tx_id = Uuid::new_v4();

        assert!(!ledger.is_transaction_processed(tx_id).unwrap());

        let result = ledger.mark_transaction_processed(tx_id);
        assert!(result.is_ok());

        assert!(ledger.is_transaction_processed(tx_id).unwrap());
    }
}

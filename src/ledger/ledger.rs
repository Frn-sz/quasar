//! Ledger module for managing accounts and transactions history.

use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use crate::{
    ledger::{error::LedgerError, interface::LedgerInterface},
    models::{Account, Transaction},
};

pub struct Ledger {
    // For now using a global lock for baseline implementation; must be optimized later.
    accounts: RwLock<HashMap<uuid::Uuid, Account>>,
    // To prevent processing the same transaction multiple times (ensure idempotency).
    processed_transactions: RwLock<HashSet<uuid::Uuid>>,
}

impl Ledger {
    fn acquire_accounts_write_lock(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, HashMap<uuid::Uuid, Account>>, LedgerError> {
        self.accounts
            .write()
            .map_err(|_| LedgerError::FailedToAcquireAccountsWriteLock)
    }

    fn acquire_accounts_read_lock(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, HashMap<uuid::Uuid, Account>>, LedgerError> {
        self.accounts
            .read()
            .map_err(|_| LedgerError::FailedToAcquireAccountsReadLock)
    }

    fn acquire_transactions_write_lock(
        &self,
    ) -> Result<std::sync::RwLockWriteGuard<'_, HashSet<uuid::Uuid>>, LedgerError> {
        self.processed_transactions
            .write()
            .map_err(|_| LedgerError::FailedToAcquireTransactionsWriteLock)
    }

    fn _acquire_transactions_read_lock(
        &self,
    ) -> Result<std::sync::RwLockReadGuard<'_, HashSet<uuid::Uuid>>, LedgerError> {
        self.processed_transactions
            .read()
            .map_err(|_| LedgerError::FailedToAcquireTransactionsReadLock)
    }
}

impl LedgerInterface for Ledger {
    fn create_account(&mut self) -> Result<uuid::Uuid, super::error::LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;

        let (account_id, account) = Account::new(vec![]);

        accounts.insert(account_id, account);

        Ok(account_id)
    }

    fn delete_account(&mut self, id: uuid::Uuid) -> Result<(), super::error::LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;

        if accounts.remove(&id).is_none() {
            return Err(LedgerError::AccountNotFound);
        }

        Ok(())
    }

    fn update_account_balance(
        &mut self,
        id: uuid::Uuid,
        amount: u64,
    ) -> Result<(), super::error::LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;

        let account = accounts.get_mut(&id).ok_or(LedgerError::AccountNotFound)?;

        account.balance += amount;

        Ok(())
    }

    fn get_account_balance(&self, id: uuid::Uuid) -> Result<u64, super::error::LedgerError> {
        let accounts = self.acquire_accounts_read_lock()?;

        let Ok(account) = accounts.get(&id).ok_or(LedgerError::AccountNotFound) else {
            return Err(LedgerError::AccountNotFound);
        };

        Ok(account.balance)
    }

    fn get_account_history(
        &self,
        id: uuid::Uuid,
    ) -> Result<Vec<crate::models::Transaction>, LedgerError> {
        let accounts = self.acquire_accounts_read_lock()?;

        let Ok(account) = accounts.get(&id).ok_or(LedgerError::AccountNotFound) else {
            return Err(LedgerError::AccountNotFound);
        };

        // Cloning for now; must be optimized later.
        Ok(account.transaction_history.clone())
    }

    fn transfer(
        &mut self,
        transaction_id: uuid::Uuid,
        source_account_id: uuid::Uuid,
        destination_account_id: uuid::Uuid,
        amount: u64,
    ) -> Result<(), LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;

        {
            let mut processed_lock = self.acquire_transactions_write_lock()?;
            if processed_lock.contains(&transaction_id) {
                // Transaction already processed; idempotency
                return Err(LedgerError::TransactionAlreadyProcessed);
            }
            // Insert the transaction ID to mark it as processed
            processed_lock.insert(transaction_id);
        }

        if source_account_id == destination_account_id {
            // No-op for transfers to self
            return Ok(());
        }

        let source_balance = accounts
            .get(&source_account_id)
            .ok_or(LedgerError::AccountNotFound)?
            .balance;

        // Check source account balance
        if source_balance < amount {
            return Err(LedgerError::InsufficientFunds);
        }

        // Checks if destination account exists
        if !accounts.contains_key(&destination_account_id) {
            return Err(LedgerError::AccountNotFound);
        }

        // Creates transaction record
        let transaction = Transaction {
            id: transaction_id,
            source_account_id,
            destination_account_id,
            amount,
            timestamp: chrono::Utc::now(),
            status: crate::models::TransactionStatus::Completed,
        };

        // This whole next section must be optimized later to avoid multiple lookups and clones.

        // Deducts amount from source account and adds transaction to history
        let source_account = accounts.get_mut(&source_account_id).unwrap(); // Safe unwrap due to previous checks
        source_account.balance -= amount;
        source_account.transaction_history.push(transaction.clone());

        // Adds amount to destination account and adds transaction to history
        let destination_account = accounts.get_mut(&destination_account_id).unwrap(); // Safe unwrap due to previous checks
        destination_account.balance += amount;
        destination_account.transaction_history.push(transaction);

        Ok(())
    }
}

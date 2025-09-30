//! Ledger module for managing accounts and transactions history.

pub mod error;
pub mod interface;

use {
    crate::{
        ledger::{error::LedgerError, interface::LedgerInterface},
        models::{Account, Transaction},
    },
    std::{
        collections::{HashMap, HashSet},
        sync::RwLock,
    },
};

pub struct Ledger {
    // For now using a global lock for baseline implementation; must be optimized later.
    accounts: RwLock<HashMap<uuid::Uuid, Account>>,
    // To prevent processing the same transaction multiple times (ensure idempotency).
    processed_transactions: RwLock<HashSet<uuid::Uuid>>,
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

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerInterface for Ledger {
    fn create_account(&mut self) -> Result<uuid::Uuid, LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;

        let (account_id, account) = Account::new(vec![]);

        accounts.insert(account_id, account);

        Ok(account_id)
    }

    fn get_account(&self, id: uuid::Uuid) -> Result<crate::models::Account, LedgerError> {
        let accounts = self.acquire_accounts_read_lock()?;

        accounts
            .get(&id)
            .cloned()
            .ok_or(LedgerError::AccountNotFound)
    }

    /// Commits account updates to the ledger.
    /// This method doesn't perform any validation to avoid race conditions.
    /// Validation must be done by the orchestrator.
    fn commit_updates(
        &mut self,
        transaction: &Transaction,
        source_account: &mut Account,
        dest_account: &mut Account,
    ) -> Result<(), LedgerError> {
        let mut accounts = self.acquire_accounts_write_lock()?;
        let mut processed_transactions = self.acquire_transactions_write_lock()?;

        accounts.insert(source_account.uuid, source_account.clone());
        accounts.insert(dest_account.uuid, dest_account.clone());
        processed_transactions.insert(transaction.id);

        Ok(())
    }

    fn is_transaction_processed(&self, transaction_id: uuid::Uuid) -> Result<bool, LedgerError> {
        let processed_transactions = self._acquire_transactions_read_lock()?;

        Ok(processed_transactions.contains(&transaction_id))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::models::{Transaction, TransactionStatus},
        chrono::Utc,
        uuid::Uuid,
    };

    #[test]
    fn test_create_account() {
        let mut ledger = Ledger::new();
        let account_id_result = ledger.create_account();
        assert!(account_id_result.is_ok());
        let account_id = account_id_result.unwrap();

        let accounts_lock = ledger.accounts.read().unwrap();
        assert!(accounts_lock.contains_key(&account_id));
    }

    #[test]
    fn test_get_existing_account() {
        let mut ledger = Ledger::new();
        let account_id = ledger.create_account().unwrap();

        let account_result = ledger.get_account(account_id);
        assert!(account_result.is_ok());
        assert_eq!(account_result.unwrap().uuid, account_id);
    }

    #[test]
    fn test_get_non_existing_account() {
        let ledger = Ledger::new();
        let non_existing_id = Uuid::new_v4();

        let account_result = ledger.get_account(non_existing_id);
        assert!(account_result.is_err());
        assert!(matches!(
            account_result.err().unwrap(),
            LedgerError::AccountNotFound
        ));
    }

    #[test]
    fn test_commit_updates_and_is_processed() {
        let mut ledger = Ledger::new();
        let source_id = ledger.create_account().unwrap();
        let dest_id = ledger.create_account().unwrap();

        let mut source_account = ledger.get_account(source_id).unwrap();
        let mut dest_account = ledger.get_account(dest_id).unwrap();
        source_account.balance = 50;
        dest_account.balance = 150;

        // Mock transaction
        let transaction = Transaction {
            id: Uuid::new_v4(),
            source_account_id: source_id,
            destination_account_id: dest_id,
            amount: 50,
            timestamp: Utc::now(),
            status: TransactionStatus::Completed,
        };

        assert!(!ledger.is_transaction_processed(transaction.id).unwrap());

        let commit_result =
            ledger.commit_updates(&transaction, &mut source_account, &mut dest_account);

        assert!(commit_result.is_ok());

        assert!(ledger.is_transaction_processed(transaction.id).unwrap());

        let final_source_balance = ledger.get_account(source_id).unwrap().balance;
        let final_dest_balance = ledger.get_account(dest_id).unwrap().balance;

        assert_eq!(final_source_balance, 50);
        assert_eq!(final_dest_balance, 150);
    }
}

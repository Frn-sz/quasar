pub mod error;
pub mod interface;
use {
    crate::{
        ledger::{error::LedgerError, interface::LedgerInterface},
        models::{Account, Key, TransferInstruction},
    },
    chrono::Utc,
    dashmap::{DashMap, DashSet},
    uuid::Uuid,
};

pub struct Ledger {
    pub accounts: DashMap<Uuid, Account>,
    // To prevent processing the same transaction multiple times (ensure idempotency).
    pub processed_transactions: DashSet<Uuid>,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new(DashMap::new(), DashSet::new())
    }
}

impl Ledger {
    pub fn new(accounts: DashMap<Uuid, Account>, processed_transactions: DashSet<Uuid>) -> Self {
        Ledger {
            accounts: accounts,
            processed_transactions,
        }
    }
}

impl LedgerInterface for Ledger {
    fn create_account(&self, keys: Vec<Key>) -> Result<Uuid, LedgerError> {
        let (account_id, account) = Account::new(keys);
        self.accounts.insert(account_id, account);
        Ok(account_id)
    }

    fn get_account(&self, id: Uuid) -> Result<Account, LedgerError> {
        match self.accounts.get(&id) {
            Some(entry) => Ok(entry.value().clone()),
            None => Err(LedgerError::AccountNotFound),
        }
    }

    fn commit_transfer(
        &self,
        transaction_id: Uuid,
        _instruction: &TransferInstruction,
        source_account: &mut Account,
        dest_account: &mut Account,
    ) -> Result<(), LedgerError> {
        // Add instruction to history
        let _timestamp = Utc::now();
        source_account.transaction_history.push(transaction_id);

        dest_account.transaction_history.push(transaction_id);

        self.accounts
            .insert(source_account.uuid, source_account.clone());

        self.accounts
            .insert(dest_account.uuid, dest_account.clone());

        self.processed_transactions.insert(transaction_id);

        Ok(())
    }

    fn is_transaction_processed(&self, transaction_id: Uuid) -> Result<bool, LedgerError> {
        Ok(self.processed_transactions.contains(&transaction_id))
    }

    fn mark_transaction_processed(&self, transaction_id: Uuid) -> Result<(), LedgerError> {
        self.processed_transactions.insert(transaction_id);
        Ok(())
    }

    fn deposit_into_account(&self, account_id: Uuid, amount: u64) -> Result<(), LedgerError> {
        let mut account = self
            .accounts
            .get_mut(&account_id)
            .ok_or(LedgerError::AccountNotFound)?;

        account.balance = account.balance.saturating_add(amount);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::models::{Key, TransferInstruction},
        uuid::Uuid,
    };

    #[test]
    fn test_create_account() {
        let ledger = Ledger::new(DashMap::new(), DashSet::new());
        let keys = vec![Key::Email("test@test.com".to_string())];
        let account_id_result = ledger.create_account(keys);
        assert!(account_id_result.is_ok());
        let account_id = account_id_result.unwrap();

        assert!(ledger.accounts.contains_key(&account_id));
        assert_eq!(ledger.accounts.get(&account_id).unwrap().keys.len(), 1);
    }

    #[test]
    fn test_get_existing_account() {
        let ledger = Ledger::new(DashMap::new(), DashSet::new());
        let account_id = ledger.create_account(vec![]).unwrap();
        let account_result = ledger.get_account(account_id);
        assert!(account_result.is_ok());
        assert_eq!(account_result.unwrap().uuid, account_id);
    }

    #[test]
    fn test_commit_transfer_and_is_processed() {
        let ledger = Ledger::new(DashMap::new(), DashSet::new());
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
        let ledger = Ledger::new(DashMap::new(), DashSet::new());
        let tx_id = Uuid::new_v4();

        assert!(!ledger.is_transaction_processed(tx_id).unwrap());

        let result = ledger.mark_transaction_processed(tx_id);
        assert!(result.is_ok());

        assert!(ledger.is_transaction_processed(tx_id).unwrap());
    }
}

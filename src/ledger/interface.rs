use uuid::Uuid;

use crate::ledger::error::LedgerError;

pub trait LedgerInterface {
    /// Creates a new account and returns its UUID.
    fn create_account(&mut self) -> Result<Uuid, LedgerError>;

    /// Deletes an account by its UUID.
    fn delete_account(&mut self, id: uuid::Uuid) -> Result<(), LedgerError>;

    /// Updates the balance of an account.
    fn update_account_balance(&mut self, id: uuid::Uuid, amount: u64) -> Result<(), LedgerError>;

    /// Gets the balance of an account.
    fn get_account_balance(&self, id: uuid::Uuid) -> Result<u64, LedgerError>;

    /// Gets the transaction history of an account.
    fn get_account_history(
        &self,
        id: uuid::Uuid,
    ) -> Result<Vec<crate::models::Transaction>, LedgerError>;

    /// Transfers an amount from one account to another.
    fn transfer(
        &mut self,
        // Needed to avoid duplicate transactions
        transaction_id: uuid::Uuid,
        from: uuid::Uuid,
        to: uuid::Uuid,
        amount: u64,
    ) -> Result<(), LedgerError>;
}

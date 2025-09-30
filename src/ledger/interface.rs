use {
    crate::{
        ledger::error::LedgerError,
        models::{Account, Transaction},
    },
    uuid::Uuid,
};

pub trait LedgerInterface {
    /// Creates a new account and returns its UUID.
    fn create_account(&mut self) -> Result<Uuid, LedgerError>;

    /// Retrieves an account by its UUID.
    fn get_account(&self, id: uuid::Uuid) -> Result<crate::models::Account, LedgerError>;

    /// Commits account updates to the ledger.
    fn commit_updates(
        &mut self,
        transaction: &Transaction,
        source_account: &mut Account,
        dest_account: &mut Account,
    ) -> Result<(), LedgerError>;

    /// Checks if a transaction has been processed.
    fn is_transaction_processed(&self, transaction_id: Uuid) -> Result<bool, LedgerError>;
}

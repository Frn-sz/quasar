use {
    crate::{
        ledger::error::LedgerError,
        models::{Account, Key, TransferInstruction},
    },
    uuid::Uuid,
};

pub trait LedgerInterface {
    /// Creates a new account with the given keys and returns its UUID.
    fn create_account(&mut self, keys: Vec<Key>) -> Result<Uuid, LedgerError>;

    /// Gets a clone of an account by its UUID.
    fn get_account(&self, id: Uuid) -> Result<Account, LedgerError>;

    /// Atomically commits the state changes for a transfer instruction.
    fn commit_transfer(
        &mut self,
        transaction_id: Uuid,
        instruction: &TransferInstruction,
        source_account: &mut Account,
        dest_account: &mut Account,
    ) -> Result<(), LedgerError>;

    /// Checks if a transaction ID has already been processed.
    fn is_transaction_processed(&self, transaction_id: Uuid) -> Result<bool, LedgerError>;

    /// Marks a transaction ID as processed.
    fn mark_transaction_processed(&mut self, transaction_id: Uuid) -> Result<(), LedgerError>;
}

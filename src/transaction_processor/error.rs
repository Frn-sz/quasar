use {crate::ledger::error::LedgerError, thiserror::Error};

#[derive(Debug, Error)]
pub enum TransactionProcessorError {
    #[error("A ledger error occurred")]
    Ledger(#[from] LedgerError),
    #[error("Transaction has already been processed")]
    TransactionAlreadyProcessed,
    #[error("Insufficient funds for the transaction")]
    InsufficientFunds,
}

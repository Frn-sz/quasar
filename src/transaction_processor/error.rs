use {crate::ledger::error::LedgerError, thiserror::Error};

#[derive(Debug, Error)]
pub enum TransactionProcessorError {
    #[error("Ledger error: {0}")]
    LedgerError(#[from] LedgerError),
    #[error("Transaction has already been processed")]
    TransactionAlreadyProcessed,
    #[error("Insufficient funds for the transaction")]
    InsufficientFunds,
    #[error("Failed to acquire ledger lock")]
    FailedToAcquireLedgerLock,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("Failed to acquire write lock on accounts")]
    FailedToAcquireAccountsWriteLock,
    #[error("Failed to acquire read lock on accounts")]
    FailedToAcquireAccountsReadLock,
    #[error("Failed to acquire write lock on transactions")]
    FailedToAcquireTransactionsWriteLock,
    #[error("Failed to acquire read lock on transactions")]
    FailedToAcquireTransactionsReadLock,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Transaction has already been processed")]
    TransactionAlreadyProcessed,
    #[error("Insufficient funds")]
    InsufficientFunds,
}

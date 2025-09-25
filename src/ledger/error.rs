pub enum LedgerError {
    FailedToAcquireAccountsWriteLock,
    FailedToAcquireAccountsReadLock,
    FailedToAcquireTransactionsWriteLock,
    FailedToAcquireTransactionsReadLock,
    AccountNotFound,
    TransactionAlreadyProcessed,
    InsufficientFunds,
}

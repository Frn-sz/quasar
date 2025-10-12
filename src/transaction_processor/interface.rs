use {crate::transaction_processor::error::TransactionProcessorError, uuid::Uuid};

#[derive(Debug, PartialEq)]
pub enum TransactionResult {
    Success,
    AccountCreated(Uuid),
    Balance(u64),
}

pub trait TransactionProcessorInterface {
    /// Processes a transaction and commits it to the ledger.
    fn process_transaction(
        &mut self,
        transaction: crate::models::Transaction,
    ) -> Result<TransactionResult, TransactionProcessorError>;
}

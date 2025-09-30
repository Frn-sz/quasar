use crate::transaction_processor::error::TransactionProcessorError;

pub trait TransactionProcessorInterface {
    /// Processes a transaction and commits it to the ledger.
    fn process_transaction(
        &mut self,
        transaction: crate::models::Transaction,
    ) -> Result<(), TransactionProcessorError>;

    /// Validates a transaction before processing.
    fn validate_transaction(
        &self,
        transaction: &crate::models::Transaction,
    ) -> Result<(), TransactionProcessorError>;

    /// Simulates a transaction without committing it to the ledger.
    fn simulate_transaction(
        &self,
        transaction: &crate::models::Transaction,
    ) -> Result<(), TransactionProcessorError>;
}

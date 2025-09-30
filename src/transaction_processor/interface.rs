use crate::transaction_processor::error::TransactionProcessorError;

pub trait TransactionProcessorInterface {
    /// Processes a transaction and commits it to the ledger.
    fn process_transaction(
        &mut self,
        transaction: crate::models::Transaction,
    ) -> Result<(), TransactionProcessorError>;
}

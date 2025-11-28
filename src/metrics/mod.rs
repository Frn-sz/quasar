use prometheus::{Counter, Histogram};

use crate::metrics::handler::{counter, histogram_fast_ops, histogram_slow_ops};
pub mod handler;
lazy_static::lazy_static!(
    pub static ref TRANSACTIONS_PROCESSED_TOTAL: Counter =
        counter("transactions_processed_total", "Total number of processed transactions");

    pub static ref TRANSACTIONS_FAILED_TOTAL: Counter =
        counter("transactions_failed_total", "Total number of failed transactions");

    pub static ref ACCOUNTS_CREATED_TOTAL: Counter =
        counter("accounts_created_total", "Total number of created accounts");


    pub static ref TRANSACTION_PROCESSING_TIME_SECONDS: Histogram =
        histogram_slow_ops("transaction_processing_time_seconds", "Total time spent processing transactions in seconds");

    pub static ref ACCOUNT_CREATION_TIME_SECONDS: Histogram =
        histogram_slow_ops("account_creation_time_seconds", "Total time spent creating accounts in seconds");

    pub static ref TRANSFER_TIME_SECONDS: Histogram =
        histogram_slow_ops("transfer_time_seconds", "Total time spent transferring funds in seconds");

    pub static ref DEPOSIT_TIME_SECONDS: Histogram =
        histogram_fast_ops("deposit_time_seconds", "Total time spent depositing funds in seconds");

    pub static ref GET_BALANCE_TIME_SECONDS: Histogram =
        histogram_fast_ops("get_balance_time_seconds", "Total time spent getting account balance in seconds");
);

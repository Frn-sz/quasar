pub mod handler;

use prometheus::{Counter, Histogram};

use crate::metrics::handler::{counter, histogram_microseconds, histogram_milliseconds};

lazy_static::lazy_static!(
    pub static ref TRANSACTIONS_PROCESSED_TOTAL: Counter =
        counter("transactions_processed_total", "Total number of processed transactions");

    pub static ref TRANSACTIONS_FAILED_TOTAL: Counter =
        counter("transactions_failed_total", "Total number of failed transactions");

    pub static ref ACCOUNTS_CREATED_TOTAL: Counter =
        counter("accounts_created_total", "Total number of created accounts");

    pub static ref TRANSACTION_PROCESSING_TIME_US: Histogram =
        histogram_microseconds("transaction_processing_time_us", "Total time spent processing transactions in microseconds");

    pub static ref ACCOUNT_CREATION_TIME_US: Histogram =
        histogram_microseconds("account_creation_time_us", "Total time spent creating accounts in microseconds");

    pub static ref TRANSFER_TIME_US: Histogram =
        histogram_microseconds("transfer_time_us", "Total time spent transferring funds in microseconds");

    pub static ref DEPOSIT_TIME_US: Histogram =
        histogram_microseconds("deposit_time_us", "Total time spent depositing funds in microseconds");

    pub static ref GET_BALANCE_TIME_US: Histogram =
        histogram_microseconds("get_balance_time_us", "Total time spent getting account balance in microseconds");
);

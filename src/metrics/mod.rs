pub mod handler;

use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static::lazy_static!(
    pub static ref TRANSACTIONS_PROCESSED_TOTAL: Counter =
        register_counter!("transactions_processed_total", "Total number of processed transactions")
            .unwrap();

    pub static ref TRANSACTIONS_FAILED_TOTAL: Counter =
        register_counter!("transactions_failed_total", "Total number of failed transactions")
            .unwrap();

    pub static ref ACCOUNTS_CREATED_TOTAL: Counter =
        register_counter!("accounts_created_total", "Total number of created accounts").unwrap();

    pub static ref TRANSACTION_PROCESSING_TIME_US: Histogram =
        register_histogram!("transaction_processing_time_us", "Total time spent processing transactions in microseconds")
            .unwrap();
);

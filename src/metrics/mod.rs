pub mod server;

use prometheus::{Counter, register_counter};

lazy_static::lazy_static!(
    pub static ref TRANSACTIONS_PROCESSED_TOTAL: Counter =
        register_counter!("transactions_processed_total", "Total number of processed transactions")
            .unwrap();
);

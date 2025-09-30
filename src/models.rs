use {
    chrono::{DateTime, Utc},
    uuid::Uuid,
};

/// Represents a possible identifier for an account.
#[derive(Debug, Clone)]
pub enum Key {
    CPF(String),
    Email(String),
    Phone(String),
    Random(String),
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub source_account_id: Uuid,
    pub destination_account_id: Uuid,
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
}

/// Account is very simplified, since we don't really care about user data
#[derive(Default, Debug, Clone)]
pub struct Account {
    pub uuid: Uuid,
    pub balance: u64,
    pub keys: Vec<Key>,
    // For now using this naive approach; must be optimized later.
    // This leads to data duplication and bad CPU cache locality.
    pub transaction_history: Vec<Transaction>,
}

impl Account {
    pub fn new(keys: Vec<Key>) -> (Uuid, Self) {
        let uuid = Uuid::new_v4();

        let account = Account {
            uuid,
            balance: 0,
            keys,
            transaction_history: vec![],
        };

        (uuid, account)
    }
}

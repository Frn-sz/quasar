use {
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

/// Represents a possible identifier for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key {
    CPF(String),
    Email(String),
    Phone(String),
    Random(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Transfer(TransferInstruction),
    CreateAccount(CreateAccountInstruction),
    Deposit(DepositInstruction),
    GetBalance(GetBalanceInstruction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub instruction: Instruction,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInstruction {
    pub source_account_id: Uuid,
    pub destination_account_id: Uuid,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountInstruction {
    pub keys: Vec<Key>,
}

impl CreateAccountInstruction {
    pub fn new(keys: Vec<Key>) -> Self {
        CreateAccountInstruction { keys }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositInstruction {
    pub destination_account_id: Uuid,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBalanceInstruction {
    pub account_id: Uuid,
}

/// Account is very simplified, since we don't really care about user data
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub uuid: Uuid,
    pub balance: u64,
    pub keys: Vec<Key>,
    // Using indirection to avoid data duplication. The vector stores transaction IDs.
    pub transaction_history: Vec<Uuid>,
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

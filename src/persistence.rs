use crate::models::{Account, Transaction};
use dashmap::{DashMap, DashSet};
use rusqlite::{Connection, Result};
use uuid::Uuid;

pub struct Persistence {
    conn: Connection,
}

impl Persistence {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let persistence = Persistence { conn };
        persistence.init_db()?;
        Ok(persistence)
    }

    fn init_db(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                uuid TEXT PRIMARY KEY,
                balance INTEGER NOT NULL,
                keys TEXT NOT NULL,
                transaction_history TEXT NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                instruction TEXT NOT NULL,
                status TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS processed_transactions (
                id TEXT PRIMARY KEY
            )",
            [],
        )?;
        Ok(())
    }

    pub fn save_state(
        &mut self,
        accounts: &DashMap<Uuid, Account>,
        transactions: &DashMap<Uuid, Transaction>,
        processed_transactions: &DashSet<Uuid>,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM accounts", [])?;
        tx.execute("DELETE FROM transactions", [])?;
        tx.execute("DELETE FROM processed_transactions", [])?;

        for account in accounts.iter() {
            let keys = serde_json::to_string(&account.keys).unwrap();
            let transaction_history = serde_json::to_string(&account.transaction_history).unwrap();

            tx.execute(
                "INSERT INTO accounts (uuid, balance, keys, transaction_history) VALUES (?1, ?2, ?3, ?4)",
                &[
                    &account.uuid.to_string(),
                    &account.balance.to_string(),
                    &keys,
                    &transaction_history,
                ],
            )?;
        }

        for transaction in transactions.iter() {
            let instruction = serde_json::to_string(&transaction.instruction).unwrap();
            let status = serde_json::to_string(&transaction.status).unwrap();
            let timestamp = transaction.timestamp.to_rfc3339();

            tx.execute(
                "INSERT INTO transactions (id, instruction, status, timestamp) VALUES (?1, ?2, ?3, ?4)",
                &[
                    &transaction.id.to_string(),
                    &instruction,
                    &status,
                    &timestamp,
                ],
            )?;
        }

        for transaction_id in processed_transactions.iter() {
            tx.execute(
                "INSERT INTO processed_transactions (id) VALUES (?1)",
                &[&transaction_id.to_string()],
            )?;
        }

        tx.commit()
    }

    pub fn load_state(&self) -> Result<(DashMap<Uuid, Account>, DashMap<Uuid, Transaction>, DashSet<Uuid>)> {
        let mut stmt = self
            .conn
            .prepare("SELECT uuid, balance, keys, transaction_history FROM accounts")?;
        let account_iter = stmt.query_map([], |row| {
            let uuid: String = row.get(0)?;
            let uuid = Uuid::parse_str(&uuid).unwrap();
            let balance: u64 = row.get(1)?;
            let keys: String = row.get(2)?;
            let transaction_history: String = row.get(3)?;

            let keys = serde_json::from_str(&keys).unwrap();
            let transaction_history = serde_json::from_str(&transaction_history).unwrap();

            Ok((
                uuid,
                Account {
                    uuid,
                    balance,
                    keys,
                    transaction_history,
                },
            ))
        })?;

        let accounts = DashMap::new();
        for account in account_iter {
            let (uuid, account) = account?;
            accounts.insert(uuid, account);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT id, instruction, status, timestamp FROM transactions")?;
        let transaction_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let id = Uuid::parse_str(&id).unwrap();
            let instruction: String = row.get(1)?;
            let status: String = row.get(2)?;
            let timestamp: String = row.get(3)?;

            let instruction = serde_json::from_str(&instruction).unwrap();
            let status = serde_json::from_str(&status).unwrap();
            let timestamp = timestamp.parse().unwrap();

            Ok((
                id,
                Transaction {
                    id,
                    instruction,
                    status,
                    timestamp,
                },
            ))
        })?;

        let transactions = DashMap::new();
        for transaction in transaction_iter {
            let (id, transaction) = transaction?;
            transactions.insert(id, transaction);
        }

        let mut stmt = self.conn.prepare("SELECT id FROM processed_transactions")?;
        let processed_transaction_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let id = Uuid::parse_str(&id).unwrap();
            Ok(id)
        })?;

        let processed_transactions = DashSet::new();
        for id in processed_transaction_iter {
            processed_transactions.insert(id?);
        }

        Ok((accounts, transactions, processed_transactions))
    }
}

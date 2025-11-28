use crate::models::Account;
use rusqlite::{Connection, Result};
use std::collections::HashMap;
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
        Ok(())
    }

    pub fn save_accounts(&mut self, accounts: &HashMap<Uuid, Account>) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM accounts", [])?;

        for account in accounts.values() {
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

        tx.commit()
    }

    pub fn load_accounts(&self) -> Result<HashMap<Uuid, Account>> {
        let mut stmt = self.conn.prepare("SELECT uuid, balance, keys, transaction_history FROM accounts")?;
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

        let mut accounts = HashMap::new();
        for account in account_iter {
            let (uuid, account) = account?;
            accounts.insert(uuid, account);
        }

        Ok(accounts)
    }
}

use uuid::Uuid;

/// Represents a possible identifier for an account.
#[derive(Debug, Clone)]
pub enum Key {
    CPF(String),
    Email(String),
    Phone(String),
    Random(String),
}

/// Account is very simplified, since we don't really care about user data
#[derive(Default, Debug, Clone)]
pub struct Account {
    pub uuid: Uuid,
    // TODO: Implement a module to deal with integer
    pub balance: i64,
    pub keys: Vec<Key>,
}

impl Account {
    pub fn new(keys: Vec<Key>) -> Self {
        Account {
            uuid: Uuid::new_v4(),
            balance: 0,
            keys,
        }
    }
}

use uuid::Uuid;

use crate::models::key::Key;

pub struct User {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub cpf: String,
    pub email: String,
    // TODO: Implement a module to deal with calcs
    pub balance: i64,
    // User PixKeys, if any
    pub keys: Vec<Key>,
}

impl User {
    pub fn new(name: String, cpf: String, email: String) -> Self {
        User {
            uuid: Uuid::new_v4(),
            name,
            cpf,
            email,
            balance: 0,
            keys: vec![],
        }
    }

    // Just for test purposes
    pub fn default() -> Self {
        User {
            uuid: Uuid::new_v4(),
            name: "".to_string(),
            cpf: "".to_string(),
            email: "".to_string(),
            balance: 0,
            keys: vec![],
        }
    }
}

// TODO: add tests here
#[cfg(test)]
mod test {}

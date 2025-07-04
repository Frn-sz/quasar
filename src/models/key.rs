// Some better way than String?
pub enum Key {
    CPF(String),
    Email(String),
    Phone(String),
    // Random key
    Generated(String),
}

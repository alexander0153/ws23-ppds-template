mod database;
mod ffi;

pub type Payload = String;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrCode {
    DbDne,
    DbExists,
    DbEnd,
    KeyNotFound,
    TxnExists,
    TxnDne,
    EntryExists,
    EntryDne,
    Deadlock,
    Failure,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyType {
    Short,
    Int,
    Varchar,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Key {
    Short(i32),
    Int(i64),
    Varchar(String),
}

mod bindings;

use crate::{
    database::{index::IdxState, transaction::TxnState},
    ErrCode, Key, KeyType, Payload,
};

pub fn create(key_type: KeyType, name: &str) -> Result<(), ErrCode> {
    todo!()
}

pub fn drop(name: &str) -> Result<(), ErrCode> {
    todo!()
}

pub fn open_index(name: &str) -> Result<IdxState, ErrCode> {
    todo!()
}

pub fn close_index(index_state: IdxState) -> Result<(), ErrCode> {
    todo!()
}

pub fn begin_transaction() -> Result<TxnState, ErrCode> {
    todo!()
}

pub fn abort_transaction(transaction_state: TxnState) -> Result<(), ErrCode> {
    todo!()
}

pub fn commit_transaction(transaction_state: TxnState) -> Result<(), ErrCode> {
    todo!()
}

pub fn get(
    index_state: &mut IdxState,
    transaction_state: Option<&mut TxnState>,
    key: Key,
) -> Result<Payload, ErrCode> {
    todo!()
}

pub fn get_next(
    index_state: &mut IdxState,
    transaction_state: Option<&mut TxnState>,
) -> Result<(Key, Payload), ErrCode> {
    todo!()
}

pub fn insert_record(
    index_state: &mut IdxState,
    transaction_state: Option<&mut TxnState>,
    key: Key,
    payload: Payload,
) -> Result<(), ErrCode> {
    todo!()
}

pub fn delete_record(
    index_state: &mut IdxState,
    transaction_state: Option<&mut TxnState>,
    key: Key,
    payload: Option<Payload>,
) -> Result<(), ErrCode> {
    todo!()
}

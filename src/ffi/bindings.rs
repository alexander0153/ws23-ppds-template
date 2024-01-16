#![allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    clippy::upper_case_acronyms
)]

use crate::{
    database::{index::IdxState, transaction::TxnState},
    ErrCode as InternalErrorCode, Key as InternalKey, KeyType as InternalKeyType,
};
use std::ffi::{c_char, CStr, CString};

/*
Do not modify this file unless absolutely necessary. Even small changes can break ABI compatibility.
*/

/// Status messages for outcomes of API calls.
#[repr(C)]
enum ErrCode {
    SUCCESS,
    DB_DNE,
    DB_EXISTS,
    DB_END,
    KEY_NOTFOUND,
    TXN_EXISTS,
    TXN_DNE,
    ENTRY_EXISTS,
    ENTRY_DNE,
    DEADLOCK,
    FAILURE,
}

impl From<InternalErrorCode> for ErrCode {
    fn from(value: InternalErrorCode) -> Self {
        match value {
            InternalErrorCode::DbDne => ErrCode::DB_DNE,
            InternalErrorCode::DbExists => ErrCode::DB_EXISTS,
            InternalErrorCode::DbEnd => ErrCode::DB_END,
            InternalErrorCode::KeyNotFound => ErrCode::KEY_NOTFOUND,
            InternalErrorCode::TxnExists => ErrCode::TXN_EXISTS,
            InternalErrorCode::TxnDne => ErrCode::TXN_DNE,
            InternalErrorCode::EntryExists => ErrCode::ENTRY_EXISTS,
            InternalErrorCode::EntryDne => ErrCode::ENTRY_DNE,
            InternalErrorCode::Deadlock => ErrCode::DEADLOCK,
            InternalErrorCode::Failure => ErrCode::FAILURE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CStringFixed<const N: usize>([u8; N]);

impl<const N: usize> From<&String> for CStringFixed<N> {
    fn from(value: &String) -> Self {
        let mut target = [0u8; N];

        let cstring = CString::new(value.clone()).expect("string should not contain null bytes");
        let source = cstring.as_bytes_with_nul();

        if source.len() > N {
            panic!("string should not be longer than {} characters", N - 1);
        }

        target[..source.len()].copy_from_slice(source);

        CStringFixed(target)
    }
}

impl<const N: usize> From<CStringFixed<N>> for String {
    fn from(value: CStringFixed<N>) -> Self {
        CStr::from_bytes_until_nul(&value.0)
            .expect("varchar value should be null-terminated")
            .to_str()
            .expect("varchar value should be a valid UTF-8 string")
            .to_owned()
    }
}

type Payload = CStringFixed<101>;

impl Payload {
    fn is_null(&self) -> bool {
        self.0[0] == 0
    }
}

/// The record information stored in an index
#[repr(C)]
struct Record {
    /// The lookup key for the record.
    pub key: Key,
    /// The value stored under that key. It will be a null-terminated C string of no more than 100 characters.
    pub payload: Payload,
}

/// Three possible key types.
#[derive(Debug)]
#[repr(C)]
enum KeyType {
    SHORT,
    INT,
    VARCHAR,
}

impl From<KeyType> for InternalKeyType {
    fn from(value: KeyType) -> Self {
        match value {
            KeyType::SHORT => InternalKeyType::Short,
            KeyType::INT => InternalKeyType::Int,
            KeyType::VARCHAR => InternalKeyType::Varchar,
        }
    }
}

/// Stores the key value, whether it is a short, an int or a varchar.
#[derive(Debug)]
#[repr(C)]
enum Key {
    SHORT(i32),
    INT(i64),
    VARCHAR(CStringFixed<129>),
}

impl From<&Key> for InternalKey {
    fn from(value: &Key) -> Self {
        match value {
            Key::SHORT(value) => InternalKey::Short(*value),
            Key::INT(value) => InternalKey::Int(*value),
            Key::VARCHAR(value) => InternalKey::Varchar(
                (*value)
                    .try_into()
                    .expect("varchar value should be a valid UTF-8 string"),
            ),
        }
    }
}

impl From<&InternalKey> for Key {
    fn from(value: &InternalKey) -> Self {
        match value {
            InternalKey::Short(value) => Key::SHORT(*value),
            InternalKey::Int(value) => Key::INT(*value),
            InternalKey::Varchar(value) => Key::VARCHAR(
                (value)
                    .try_into()
                    .expect("varchar value should not contain null bytes"),
            ),
        }
    }
}

/// Creates a new index data structure to be used by any thread.
///
/// @param type specifies what type of key the index will use
/// @param name a unique name to be used to identify this index in any process
/// @return ErrCode
/// SUCCESS if successfully created index.
/// DB_EXISTS if index with specified name already exists.
/// FAILURE if could not create index for some other reason.
#[no_mangle]
extern "C" fn create(type_: KeyType, name: *mut c_char) -> ErrCode {
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect("name should be a valid UTF-8 string");

    match super::create(type_.into(), name) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Drops an existing index.
///
/// @param name an existing index
/// @return ErrCode
/// SUCCESS if successfully dropped index.
/// FAILURE index doesn't exist or some other reason.
#[no_mangle]
extern "C" fn drop(name: *mut c_char) -> ErrCode {
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect("name should be a valid UTF-8 string");

    match super::drop(name) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Opens a specific index data structure to be used by this thread.
///
/// @param name the unique name specifying the index being opened
/// @param idxState returns the state handle for the index being opened
/// @return ErrCode
/// SUCCESS if successfully opened index.
/// DB_DNE if the name given does not have an associated DB that has been create()d.
/// FAILURE if DB exists but could not be opened for some other reason.
#[no_mangle]
extern "C" fn openIndex(name: *const c_char, idxState: *mut *mut IdxState) -> ErrCode {
    let name = unsafe { CStr::from_ptr(name) }
        .to_str()
        .expect("name should be a valid UTF-8 string");

    match super::open_index(name) {
        Ok(index) => {
            let index_ptr = Box::into_raw(Box::new(index));
            unsafe { *idxState = index_ptr };
            ErrCode::SUCCESS
        }
        Err(error) => ErrCode::from(error),
    }
}

/// Terminate use of current index by this thread.
///
/// @param idxState The state variable for the index being closed
/// @return ErrCode
/// SUCCESS if succesfully closed index.
/// DB_DNE is the DB never existed or was already closed by someone else.
/// FAILURE if could not close DB for some other reason.
#[no_mangle]
extern "C" fn closeIndex(idxState: *mut IdxState) -> ErrCode {
    match super::close_index(unsafe { *Box::from_raw(idxState) }) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Signals the beginning of a transaction.  Each thread can have only
/// one outstanding transaction running at a time.
///
/// @param txn Returns the transaction state for the new transaction.
/// @return ErrCode
/// SUCCESS if successfully began transaction.
/// TXN_EXISTS if there is already a transaction begun for this thread.
/// DEADLOCK if this transaction had to be aborted because of deadlock.
/// FAILURE if could not begin transaction for some other reason.
#[no_mangle]
extern "C" fn beginTransaction(txn: *mut *mut TxnState) -> ErrCode {
    match super::begin_transaction() {
        Ok(transaction) => {
            let transaction_ptr = Box::into_raw(Box::new(transaction));
            unsafe { *txn = transaction_ptr };
            ErrCode::SUCCESS
        }
        Err(error) => ErrCode::from(error),
    }
}

/// Forces the current transaction to abort, rolling back all changes
/// made during the course of the transaction.
///
/// @param txn The state variable for the transaction being aborted.
/// @return ErrCode
/// SUCCESS if successfully aborted transaction.
/// TXN_DNE if there was no transaction to abort.
/// DEADLOCK if the abort failed because of deadlock.
/// FAILURE if could not abort transaction for some other reason.
#[no_mangle]
extern "C" fn abortTransaction(txn: *mut TxnState) -> ErrCode {
    match super::abort_transaction(unsafe { *Box::from_raw(txn) }) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Signals the end of the current transaction, committing
/// all changes created in the transaction.
///
/// @param txn The state variable for the transaction being committed.
/// @return ErrCode
/// SUCCESS if successfully ended transaction.
/// TXN_DNE if there was no transaction currently open.
/// DEADLOCK if this transaction could not be closed because of deadlock.
/// FAILURE if could not end transaction for some other reason.
#[no_mangle]
extern "C" fn commitTransaction(txn: *mut TxnState) -> ErrCode {
    match super::commit_transaction(unsafe { *Box::from_raw(txn) }) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Retrieve the first record associated with the given key value; if
/// more than one record exists with this key, return the first record
/// with this key. Contents of the retrieved record are copied into
/// the user supplied Record structure.
///
/// Records with the same key may be returned in any order, but it must
/// be that if there are n records with the same key k, a call to get
/// followed by n-1 calls to getNext will return all n records with key k.
///
/// If get returns KEY_NOTFOUND for a key k, the caller may invoke getNext
/// to find the first key after key k.
///
/// @param idxState The state variable for this thread
/// @param txn The transaction state to be used (or NULL if not in a transaction)
/// @param record Record containing the key being retrieved, into which the
/// payload is copied.
/// @return ErrCode
/// SUCCESS if successfully retrieved and returned unique record.
/// KEY_NOTFOUND if specified key value was not found in the DB.
/// DEADLOCK if this call could not complete because of deadlock.
/// FAILURE if could not retrieve unique record for some other reason.
#[no_mangle]
extern "C" fn get(idxState: *mut IdxState, txn: *mut TxnState, record: *mut Record) -> ErrCode {
    let record = unsafe { record.as_mut() }.expect("record should not be null");

    match super::get(
        unsafe { idxState.as_mut() }.expect("idxState should not be null"),
        unsafe { txn.as_mut() },
        (&record.key).into(),
    ) {
        Ok(payload) => {
            record.payload = (&payload).into();
            ErrCode::SUCCESS
        }
        Err(error) => ErrCode::from(error),
    }
}

/// Retrieve the record following the previous record retrieved by get or
/// getNext. If no such call has occurred since the current transaction
/// began, or if this is called from outside of a transaction, this
/// returns the first record in the index. Records are ordered in ascending
/// order by key.  Records with the same key but different payloads
/// may be returned in any order.
///
/// If get returned KEY_NOT_FOUND for a key k, invoking getNext will
/// return the first key after k.
///
/// If the index is closed and reopened, or a new transaction has begun
/// since any previous call of get or getNext, getNext returns the first
/// record in the index.
///
///
/// @param idxState The state variable for the index whose next Record
/// is to be returned
/// @param txn The transaction state to be used (or NULL if not in a transaction)
/// @param record Record through which the next key/payload pair is returned
/// @return ErrCode
/// SUCCESS if successfully retrieved and returned the next record in the DB.
/// DB_END if reached the end of the DB.
/// DEADLOCK if this call could not complete because of deadlock.
/// FAILURE if could not retrieve next record for some other reason.
#[no_mangle]
extern "C" fn getNext(idxState: *mut IdxState, txn: *mut TxnState, record: *mut Record) -> ErrCode {
    let record: &mut Record = unsafe { record.as_mut() }.expect("record should not be null");

    match super::get_next(
        unsafe { idxState.as_mut() }.expect("idxState should not be null"),
        unsafe { txn.as_mut() },
    ) {
        Ok((next_key, next_payload)) => {
            record.key = Key::from(&next_key);
            record.payload = (&next_payload).into();
            ErrCode::SUCCESS
        }
        Err(error) => ErrCode::from(error),
    }
}

/// Insert a payload associated with the given key. An identical key can
/// be used multiple times, but only with unique payloads.  If this is
/// called from outside of a transaction, it should commit immediately.
/// Records in an index are ordered in ascending order by key.  Records
/// with the same key may be stored in any order.
///
/// The implementation is responsible for making a copy of payload
/// (e.g., it may not assume that the payload pointer continues
/// to be valid after this routine returns.)
///
/// @param idxState The state variable for this thread
/// @param txn The transaction state to be used (or NULL if not in a transaction)
/// @param k key value for insert
/// @param payload Pointer to the beginning of the payload string
/// @return ErrCode
/// SUCCESS if successfully inserted record into DB.
/// ENTRY_EXISTS if identical record already exists in DB.
/// DEADLOCK if this call could not complete because of deadlock.
/// FAILURE if could not insert entry for some other reason.
#[no_mangle]
extern "C" fn insertRecord(
    idxState: *mut IdxState,
    txn: *mut TxnState,
    k: *mut Key,
    payload: *const Payload,
) -> ErrCode {
    match super::insert_record(
        unsafe { idxState.as_mut() }.expect("idxState should not be null"),
        unsafe { txn.as_mut() },
        unsafe { k.as_ref() }.expect("k should not be null").into(),
        (*unsafe { payload.as_ref() }.expect("payload should not be null")).into(),
    ) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

/// Remove the record associated with the given key from the index
/// structure.  If a payload is specified in the Record, then the
/// key/payload pair specified is removed. Otherwise, the payload pointer
/// is a length 0 string and all records with the given key are removed from the
/// database.  If this is called from outside of a transaction, it should
/// commit immediately.
///
/// @param idxState The state variable for this thread
/// @param txn The transaction state to be used (or NULL if not in a transaction)
/// @param record Record struct containing a Key and a char* payload
/// (or NULL pointer) describing what is to be deleted
/// @return ErrCode
/// SUCCESS if successfully deleted record from DB.
/// ENTRY_DNE if the specified key/payload pair could not be found in the DB.
/// KEY_NOTFOUND if the specified key could not be found in the DB, with only the key specified.
/// DEADLOCK if this call could not complete because of deadlock.
/// FAILURE if could not delete record for some other reason.
#[no_mangle]
extern "C" fn deleteRecord(
    idxState: *mut IdxState,
    txn: *mut TxnState,
    record: *mut Record,
) -> ErrCode {
    let record = unsafe { record.as_ref() }.expect("record should not be null");

    let key = (&record.key).into();
    let payload = if record.payload.is_null() {
        None
    } else {
        Some(record.payload.into())
    };

    match super::delete_record(
        unsafe { idxState.as_mut() }.expect("idxState should not be null"),
        unsafe { txn.as_mut() },
        key,
        payload,
    ) {
        Ok(_) => ErrCode::SUCCESS,
        Err(error) => ErrCode::from(error),
    }
}

use crate::prelude::*;
use redb::{AccessGuard, Key, Range, Table, TableDefinition, Value};
use std::cell::{Ref, RefCell};

pub const TABLE: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");

pub fn drop<'txn, K: Key + 'static, V: Value + 'static>(
    table: &RefCell<Table<K, V>>,
    element: (AccessGuard<'_, K>, AccessGuard<'_, V>),
) -> Result<()> {
    table.borrow_mut().remove(element.0.value()).unwrap();
    Ok(())
}

pub fn remove_duplicates(
    table: &Ref<Table<i64, Vec<u8>>>,
    range: Range<i64, Vec<u8>>,
    payload: Vec<u8>,
    dupes: i32,
) -> () {
    let cursor;
    if dupes > 0 {
        let cursor = range
            .rev()
            .take(dupes as usize)
            .filter(|entry| entry.as_ref().unwrap().1.value() == payload);
    } else if dupes < 0 {
        let cursor = range
            .take(dupes as usize)
            .filter(|entry| entry.as_ref().unwrap().1.value() == payload);
    }
    cursor.for_each(|entry| {
        table.remove(entry.unwrap().0.value()).unwrap();
    });
}

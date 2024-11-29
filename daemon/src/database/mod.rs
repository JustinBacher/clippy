mod schema;
pub mod testing;

use std::{cmp::Ordering::*, collections::HashSet};

use anyhow::Result;
use camino::Utf8Path;

pub use crate::database::schema::{
    transaction::{RTransaction, RwTransaction},
    Builder, ClipEntry, Database, ToInput, MODELS,
};
pub trait TableLen<'txn, T: ToInput> {
    fn length(&self) -> Result<u64>;
}

impl<'txn> TableLen<'txn, ClipEntry> for RTransaction<'txn> {
    fn length(&self) -> Result<u64> {
        Ok(self.len().primary::<ClipEntry>()?)
    }
}
impl<'txn> TableLen<'txn, ClipEntry> for RwTransaction<'txn> {
    fn length(&self) -> Result<u64> {
        Ok(self.len().primary::<ClipEntry>()?)
    }
}

pub fn get_db(path: &Utf8Path) -> Result<native_db::Database> {
    let db = Builder::new().create(&MODELS, path)?;
    let tx = db.rw_transaction()?;
    tx.migrate::<ClipEntry>()?;
    tx.commit()?;

    Ok(db)
}

pub fn remove_duplicates(db: &Database, duplicates: i64) -> Result<()> {
    let rtx = db.r_transaction()?;
    let wtx = db.rw_transaction()?;
    let it = rtx.scan().primary::<ClipEntry>()?;
    let cursor = it.all()?;
    let mut seen = HashSet::<Vec<u8>>::new();

    let filtered: Box<dyn Iterator<Item = ClipEntry>> = match duplicates.cmp(&0) {
        Greater => Box::new(cursor.take(duplicates as usize).flatten()),
        Less => Box::new(cursor.rev().take(duplicates.unsigned_abs() as usize).flatten()),
        Equal => Box::new(cursor.rev().flatten()),
    };

    for entry in filtered {
        if !seen.insert(entry.payload.to_vec()) {
            wtx.remove(entry).ok();
        }
    }

    Ok(wtx.commit()?)
}

pub fn ensure_db_size(db: &Database, limit: u64) -> Result<()> {
    let tx = db.rw_transaction()?;
    tx.scan()
        .primary::<ClipEntry>()?
        .all()?
        .take(limit as usize)
        .flatten()
        .for_each(|entry| {
            tx.remove(entry).unwrap();
        });
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
pub mod test {

    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use shortcut_assert_fs::TmpFs;
    use testing::{fill_db_and_test, get_db_contents, FillWith};

    use super::*;
    use crate::{
        database::schema::{ClipEntry, Database},
        utils::random_str,
    };

    #[test]
    fn it_removes_dupes_oldest() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, _| {
            remove_duplicates(db, 10)?;
            let tx = db.r_transaction()?;

            assert_eq!(tx.length()?, 12);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn it_removes_dupes_newest() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, _| {
            remove_duplicates(db, -10)?;
            let tx = db.r_transaction()?;

            assert_eq!(tx.length()?, 12);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn it_removes_all_dupes() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, before| {
            remove_duplicates(db, 0)?;
            let tx = db.r_transaction()?;
            let it = tx.scan().primary::<ClipEntry>()?;
            let mut cursor = it.all()?;

            let a_first = cursor.next().unwrap()?.payload;
            let a_second = cursor.next().unwrap()?.payload;
            let a_last = cursor.next().unwrap()?.payload;

            let b_first = before.get(1).unwrap();
            let b_last = before.get(18).unwrap();

            assert_eq!(tx.length()?, 3);
            assert_eq!(b_first, &a_first);
            assert_eq!(b_last, &a_second);
            assert_eq!(dupe.as_bytes(), a_last);
            Ok(())
        })
        .unwrap();
    }
}

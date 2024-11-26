mod schema;

use std::{cmp::Ordering::*, collections::HashSet};

use anyhow::Result;
use camino::Utf8Path;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

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

#[derive(EnumIter, Serialize, Deserialize, ValueEnum, Copy, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LinuxShells {
    Bash,
    Fish,
    Zsh,
}
pub fn get_db(path: &str) -> Result<native_db::Database> {
    let db = Builder::new().create(&MODELS, path)?;
    let tx = db.rw_transaction()?;
    tx.migrate::<ClipEntry>()?;
    tx.commit()?;

    Ok(db)
}

pub fn remove_duplicates(db: &Database, duplicates: i64) -> Result<()> {
    let rtx = db.r_transaction()?;
    let wtx = db.rw_transaction()?;
    let mut seen = HashSet::<Vec<u8>>::new();

    match duplicates.cmp(&0) {
        Greater => {
            rtx.scan()
                .primary::<ClipEntry>()?
                .all()?
                .take(duplicates as usize)
                .flatten()
                .filter(|entry| !seen.insert(entry.payload.to_vec()))
                .for_each(|entry| {
                    wtx.remove(entry).ok();
                });
        },
        Less => {
            rtx.scan()
                .primary::<ClipEntry>()?
                .all()?
                .rev()
                .take(duplicates.unsigned_abs() as usize)
                .flatten()
                .filter(|entry| !seen.insert(entry.payload.to_vec()))
                .for_each(|entry| {
                    wtx.remove(entry).ok();
                });
        },
        Equal => {
            rtx.scan()
                .primary::<ClipEntry>()?
                .all()?
                .rev()
                .flatten()
                .filter(|entry| !seen.insert(entry.payload.to_vec()))
                .for_each(|entry| {
                    wtx.remove(entry).ok();
                });
        },
    }
    wtx.commit()?;
    Ok(())
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

    use super::*;
    use crate::{
        database::schema::{ClipEntry, Database},
        utils::random_str,
    };

    pub enum FillWith<'a> {
        Dupes(&'a str),
        Random,
        DupesRandomEnds(&'a str),
    }

    pub fn get_db_contents(db: &Database) -> Result<Vec<Vec<u8>>> {
        let contents = db
            .r_transaction()?
            .scan()
            .primary::<ClipEntry>()?
            .all()?
            .flatten()
            .map(|entry| entry.payload)
            .collect_vec();

        Ok(contents)
    }

    pub fn fill_db_and_test<F>(fill: FillWith, amount: i64, func: F) -> Result<()>
    where
        F: FnOnce(&Database, Vec<Vec<u8>>) -> Result<()>,
    {
        let tf = TmpFs::new()?;
        let path = tf.path("test");
        let db = get_db(Utf8Path::new(path.as_str()))?;
        let mut all_items = Vec::<Vec<u8>>::new();

        for i in 0..20 {
            let dummy = match fill {
                FillWith::Dupes(dupe) => dupe,
                FillWith::Random => &random_str(7),
                FillWith::DupesRandomEnds(dupe) => match i {
                    1 => &random_str(7),
                    i if ![1, amount - 2].contains(&i) => dupe,
                    _ => &random_str(7),
                },
            };

            let tx = db.rw_transaction()?;
            {
                tx.insert(ClipEntry::new(payload))?;
            }
            tx.commit()?;

            all_items.push(dummy.as_bytes().to_vec());
        }

        func(&db, all_items)
    }

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

mod schema;

use std::{cmp::Ordering::*, collections::HashSet};

use anyhow::Result;
use itertools::Either::{Left, Right};

use crate::cli::ClippyCli;
pub use crate::database::schema::{
    transaction::{query::PrimaryScanIterator, RTransaction, RwTransaction},
    Builder, ClipEntry, Database, ToInput, MODELS,
};

pub trait EasyLength<'txn, T: ToInput> {
    fn length(&self) -> Result<u64>;
}

impl<'txn> EasyLength<'txn, ClipEntry> for RTransaction<'txn> {
    fn length(&self) -> Result<u64> {
        Ok(self.len().primary::<ClipEntry>()?)
    }
}
impl<'txn> EasyLength<'txn, ClipEntry> for RwTransaction<'txn> {
    fn length(&self) -> Result<u64> {
        Ok(self.len().primary::<ClipEntry>()?)
    }
}

pub fn get_db(args: &ClippyCli) -> Result<native_db::Database> {
    let db = Builder::new().create(&MODELS, &args.db_path)?;

    let tx = db.rw_transaction()?;
    tx.migrate::<ClipEntry>()?;
    tx.commit()?;

    Ok(db)
}

#[allow(clippy::iter_skip_zero)]
pub fn remove_duplicates(db: &Database, duplicates: i64) -> Result<()> {
    let tx = db.rw_transaction()?;
    {
        let it = tx.scan().primary()?;
        let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
        let mut seen = HashSet::<String>::new();

        match duplicates.cmp(&0) {
            Greater => Left(cursor.rev().skip(tx.length()? as usize - duplicates as usize)),
            Less => Right(cursor.skip(duplicates.unsigned_abs() as usize)),
            Equal => Left(cursor.rev().skip(0)),
        }
        .flatten()
        .for_each(|entry| {
            if !seen.insert(entry.payload.to_string()) {
                tx.remove(entry).ok();
            }
        });
    }
    tx.commit()?;

    Ok(())
}

pub fn ensure_db_size(db: &Database, limit: u64) -> Result<()> {
    let tx = db.rw_transaction()?;
    {
        let it = tx.scan().primary()?;
        let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
        cursor
            .rev()
            .take(tx.length()? as usize - limit as usize)
            .flatten()
            .for_each(|entry| {
                tx.remove(entry).unwrap();
            })
    }
    tx.commit()?;

    Ok(())
}

#[cfg(test)]
pub mod test {
    use std::fs;

    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use scopeguard::defer;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::{
        commands::store::store,
        database::schema::{ClipEntry, Database},
        utils::random_str,
    };

    pub enum FillWith<'a> {
        Dupes(&'a str),
        Random,
        DupesRandomEnds(&'a str),
    }

    pub fn get_db_contents(db: &Database) -> Result<Vec<String>> {
        let tx = db.r_transaction()?;
        let it = tx.scan().primary()?;
        let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
        Ok(cursor.flatten().map(|entry| entry.payload).collect_vec())
    }

    pub fn fill_db_and_test<F>(fill: FillWith, amount: i64, func: F) -> Result<()>
    where
        F: FnOnce(&Database, Vec<String>) -> Result<()>,
    {
        let tmp = NamedTempFile::new()?.into_temp_path();
        let path = tmp.to_str().unwrap().to_string();
        tmp.close()?;
        let db = Builder::new().create(&MODELS, &path)?;
        defer!(fs::remove_file(path).unwrap());
        let mut all_items = Vec::<String>::new();

        for i in 0..20 {
            let dummy = match fill {
                FillWith::Dupes(dupe) => dupe.to_string(),
                FillWith::Random => random_str(7),
                FillWith::DupesRandomEnds(dupe) => match i {
                    1 => random_str(7),
                    i if ![1, amount - 2].contains(&i) => dupe.to_string(),
                    _ => random_str(7),
                },
            };

            store(&db, &String::into_bytes(dummy.clone()))?;
            all_items.push(dummy);
        }

        func(&db, all_items)
    }

    #[test]
    fn it_removes_dupes_oldest() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, _: Vec<String>| {
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
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, _: Vec<String>| {
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
        fill_db_and_test(
            FillWith::DupesRandomEnds(dupe),
            20,
            |db, before: Vec<String>| {
                remove_duplicates(db, 0)?;
                let tx = db.r_transaction()?;
                let it = tx.scan().primary()?;
                let mut cursor: PrimaryScanIterator<ClipEntry> = it.all()?;

                let a_first = cursor.next().unwrap()?.payload;
                let a_second = cursor.next().unwrap()?.payload;
                let a_last = cursor.next().unwrap()?.payload;

                let b_first = before.get(1).unwrap();
                let b_last = before.get(18).unwrap();

                assert_eq!(tx.length()?, 3);
                assert_eq!(b_first, &a_first);
                assert_eq!(b_last, &a_second);
                assert_eq!(dupe, a_last);
                Ok(())
            },
        )
        .unwrap();
    }
}

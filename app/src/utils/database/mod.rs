pub mod clip;

use std::{cmp::Ordering::*, collections::HashSet};

use itertools::Either::{Left, Right};
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};

use crate::prelude::Result;

pub const TABLE_DEF: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");

#[allow(clippy::iter_skip_zero)]
pub fn remove_duplicates(db: &Database, duplicates: i32) -> Result<()> {
    let read_tx = db.begin_read()?;
    let write_tx = db.begin_write()?;

    {
        let read_table = read_tx.open_table(TABLE_DEF)?;
        let mut table = write_tx.open_table(TABLE_DEF)?;
        let cursor = read_table.iter()?;
        let mut seen = HashSet::<Vec<u8>>::new();

        match duplicates.cmp(&0) {
            Greater => Left(cursor.skip(read_table.len()? as usize - duplicates as usize)),
            Less => Right(cursor.rev().skip(duplicates.unsigned_abs() as usize)),
            Equal => Left(cursor.skip(0)),
        }
        .flatten()
        .for_each(|(date, payload)| {
            if !seen.insert(payload.value()) {
                table.remove(date.value()).ok();
            }
        });
    }

    write_tx.commit()?;
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
    use crate::{commands::store::store, utils::random_str};

    pub enum FillWith<'a> {
        Dupes(&'a str),
        Random,
        DupesRandomEnds(&'a str),
    }

    pub fn get_db_contents(db: &Database) -> Result<Vec<Vec<u8>>> {
        let contents = db
            .begin_read()?
            .open_table(TABLE_DEF)?
            .iter()?
            .flatten()
            .map(|entry| entry.1.value())
            .collect_vec();
        Ok(contents)
    }

    pub fn fill_db_and_test<F>(fill: FillWith, amount: i64, func: F) -> Result<()>
    where
        F: FnOnce(&Database, Vec<Vec<u8>>) -> Result<()>,
    {
        let tmp = NamedTempFile::new()?.into_temp_path();
        let path = tmp.to_str().unwrap().to_string();
        tmp.close()?;
        let db = Database::create(&path)?;
        defer!(fs::remove_file(path).unwrap());
        let mut all_items = Vec::<Vec<u8>>::new();

        for i in 0..20 {
            let dummy = String::into_bytes(match fill {
                FillWith::Dupes(dupe) => dupe.to_string(),
                FillWith::Random => random_str(7),
                FillWith::DupesRandomEnds(dupe) => match i {
                    1 => random_str(7),
                    i if ![1, amount - 1].contains(&i) => dupe.to_string(),
                    _ => random_str(7),
                },
            });

            store(&db, dummy.to_vec())?;
            all_items.push(dummy.to_vec());
        }

        func(&db, all_items)
    }

    #[test]
    fn it_removes_dupes_oldest() {
        let dupe = "asdf";
        fill_db_and_test(
            FillWith::DupesRandomEnds(dupe),
            20,
            |db, before: Vec<Vec<u8>>| {
                remove_duplicates(db, 10)?;
                let table = db.begin_read()?.open_table(TABLE_DEF)?;

                let a_first = table.first()?.unwrap().1.value();
                let a_last = table.last()?.unwrap().1.value();

                let b_first = before.get(1).unwrap();
                let b_last = before.last().unwrap();

                assert_eq!(table.len()?, 12);
                assert_eq!(b_first, &a_first);
                assert_eq!(b_last, &a_last);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn it_removes_dupes_newest() {
        let dupe = "asdf";
        fill_db_and_test(
            FillWith::DupesRandomEnds(dupe),
            20,
            |db, before: Vec<Vec<u8>>| {
                remove_duplicates(db, -10)?;
                let table = db.begin_read()?.open_table(TABLE_DEF)?;

                let a_first = table.first()?.unwrap().1.value();
                let a_last = table.last()?.unwrap().1.value();

                let b_first = before.get(1).unwrap();
                let b_last = before.last().unwrap();

                assert_eq!(table.len()?, 12);
                assert_eq!(b_first, &a_first);
                assert_eq!(b_last, &a_last);
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn it_removes_all_dupes() {
        let dupe = "asdf";
        fill_db_and_test(
            FillWith::DupesRandomEnds(dupe),
            20,
            |db, before: Vec<Vec<u8>>| {
                remove_duplicates(db, 0)?;

                let table = db.begin_read()?.open_table(TABLE_DEF)?;
                let mut cursor = table.iter()?;

                let a_first = cursor.next().unwrap()?.1.value();
                let a_second = cursor.next().unwrap()?.1.value();
                let a_last = cursor.next().unwrap()?.1.value();

                let b_first = before.get(1).unwrap();
                let b_last = before.get(18).unwrap();

                assert_eq!(table.len()?, 3);
                assert_eq!(b_first, &a_first);
                assert_eq!(b_last, &a_second);
                assert_eq!(dupe.bytes().collect_vec(), a_last);
                Ok(())
            },
        )
        .unwrap();
    }
}

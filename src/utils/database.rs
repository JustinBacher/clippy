use crate::prelude::Result;
use itertools::Either::{Left, Right};
use redb::{Database, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashMap;

pub const TABLE_DEF: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");

pub fn remove_duplicates(db: &Database, duplicates: i32) -> Result<()> {
    let read_tx = db.begin_read()?;
    let write_tx = db.begin_write()?;

    {
        let read_table = read_tx.open_table(TABLE_DEF)?;
        let mut table = write_tx.open_table(TABLE_DEF)?;

        let cursor = read_table.iter()?;
        let mut seen = HashMap::<Vec<u8>, Option<i64>>::new();

        match duplicates.cmp(&0) {
            Greater => Left(cursor.rev().take(duplicates as usize)),
            Less => Right(cursor.take(duplicates.unsigned_abs() as usize)),
            Equal => Right(cursor.take(read_table.len()? as usize - 1)),
        }
        .flatten()
        .map(|(k, v)| (k.value(), v.value()))
        .for_each(|(date, payload)| {
            seen.entry(payload)
                .and_modify(|dupe| {
                    table.remove(date).unwrap();
                    dupe.take().and_then(|old| table.remove(old).ok());
                })
                .or_insert(Some(date));
        });
    }

    write_tx.commit()?;
    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::commands::store::store;
    use itertools::Itertools;
    use pretty_assertions::assert_eq;
    use rand::{distributions::Alphanumeric, Rng};
    use scopeguard::defer;
    use std::{fs, ops::FnOnce};
    use tempfile::NamedTempFile;

    pub enum FillWith<'a> {
        Random,
        Dupes(&'a str),
    }

    pub fn get_random_string() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect()
    }

    pub fn fill_db_and_test<F>(fill: FillWith, func: F) -> Result<()>
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
            let dummy = &match fill {
                FillWith::Random => get_random_string().into_bytes(),
                FillWith::Dupes(dupe) => match i {
                    1 | 18 => get_random_string().into_bytes(),
                    _ => dupe.to_string().into_bytes(),
                },
            };

            store(&db, dummy.to_vec())?;
            all_items.push(dummy.to_vec());
        }

        func(&db, all_items)
    }

    #[test]
    fn it_removes_dupes_right() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::Dupes(dupe), |db, before: Vec<Vec<u8>>| {
            remove_duplicates(db, 10)?;
            let table = db.begin_read()?.open_table(TABLE_DEF)?;

            let a_first = table.first()?.unwrap().1.value();
            let a_last = table.last()?.unwrap().1.value();

            let b_first = before.first().unwrap();
            let b_last = before.get(18).unwrap();

            assert_eq!(table.len()?, 11);
            assert_eq!(b_first, &a_first);
            assert_eq!(b_last, &a_last);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn it_removes_dupes_left() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::Dupes(dupe), |db, before: Vec<Vec<u8>>| {
            remove_duplicates(db, -10)?;
            let table = db.begin_read()?.open_table(TABLE_DEF)?;

            let a_first = table.first()?.unwrap().1.value();
            let a_last = table.last()?.unwrap().1.value();

            let b_first = before.get(1).unwrap();
            let b_last = before.last().unwrap();

            assert_eq!(table.len()?, 11);
            assert_eq!(b_first, &a_first);
            assert_eq!(b_last, &a_last);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn it_removes_all_dupes() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::Dupes(dupe), |db, before: Vec<Vec<u8>>| {
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
        })
        .unwrap();
    }
}

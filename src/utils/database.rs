use crate::prelude::Result;
use camino::Utf8PathBuf;
use itertools::{Either, Itertools};
use redb::{Database, ReadableTable, TableDefinition};
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    iter::{Rev, Take},
};

pub const TABLE_DEF: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");

// I'm using this because I want to ignore the fact that
// using iter functions return different types.
type Reversible<T> = Either<Take<Rev<T>>, Take<T>>;

pub fn remove_duplicates(db_path: &Utf8PathBuf, duplicates: i32, max: usize) -> Result<()> {
    let db = Database::open(db_path)?;
    let read_tx = db.begin_read()?;
    let write_tx = db.begin_write()?;

    {
        let read_table = read_tx.open_table(TABLE_DEF)?;
        let cursor = read_table.iter()?;
        let mut write_table = write_tx.open_table(TABLE_DEF)?;

        match duplicates.cmp(&0) {
            Greater => Reversible::Right(cursor.take(duplicates as usize)),
            Less => Reversible::Left(cursor.rev().take(duplicates as usize)),
            Equal => Reversible::Right(cursor.take(max)),
        }
        .duplicates_by(|entry| entry.as_ref().unwrap().1.value())
        .for_each(|entry| {
            write_table.remove(entry.unwrap().0.value()).unwrap();
        });
    }

    write_tx.commit()?;
    Ok(())
}

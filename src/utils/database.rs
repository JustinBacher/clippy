use crate::prelude::*;
use camino::Utf8PathBuf;
use itertools::{Either, Itertools};
use redb::{Database, ReadableTable, TableDefinition};
use std::{
    cmp::Ordering::{Equal, Greater, Less},
    iter::{Rev, Take},
};

pub const TABLE_DEF: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");

type Reversible<T> = Either<Rev<T>, T>;
type CanTake<T> = Either<Take<Reversible<T>>, Reversible<T>>;

pub fn remove_duplicates(db_path: &Utf8PathBuf, duplicates: i32) -> Result<()> {
    let db = Database::open(&db_path)?;
    let read_tx = db.begin_write()?;
    let write_tx = db.begin_write()?;

    {
        let read_table = read_tx.open_table(TABLE_DEF)?;
        let mut write_table = write_tx.open_table(TABLE_DEF)?;
        let dupes = duplicates.cmp(&0);

        let cursor = match dupes {
            Greater => Reversible::Left(read_table.iter()?.rev()),
            Less | Equal => Reversible::Right(read_table.iter()?),
        };

        match dupes {
            Equal => CanTake::Right(cursor),
            _ => CanTake::Left(cursor.take(duplicates as usize)),
        }
        .duplicates_by(|entry| entry.as_ref().unwrap().1.value())
        .for_each(|entry| {
            write_table.remove(entry.unwrap().0.value()).unwrap();
        });
    }

    write_tx.commit()?;
    Ok(())
}

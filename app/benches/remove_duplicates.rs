use std::{
    cmp::Ordering::{Equal, Greater, Less},
    collections::HashSet,
    fs,
    hint::black_box,
};

use chrono::Local;
use clippy::{prelude::Result, utils::database::TABLE_DEF};
use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Either::{Left, Right};
use rand::{distributions::Alphanumeric, Rng};
use redb::{Database, ReadableTable, ReadableTableMetadata};
use scopeguard::defer;
use tempfile::NamedTempFile;

pub fn get_random_string() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect()
}

fn create_and_fill_db<F>(amount: usize, func: F) -> Result<()>
where
    F: FnOnce(&Database) -> Result<()>,
{
    let tmp = NamedTempFile::new().unwrap().into_temp_path();
    let path = tmp.to_str().unwrap().to_string();
    tmp.close()?;
    defer!(fs::remove_file(&path).unwrap());
    let db = Database::create(&path)?;
    for i in 0..amount {
        let tx = db.begin_write()?;
        {
            tx.open_table(TABLE_DEF)?.insert(
                Local::now().timestamp_micros(),
                match i % 10 {
                    0 => get_random_string().into_bytes(),
                    _ => "asdf".to_string().into_bytes(),
                },
            )?;
        }
        tx.commit()?;
    }
    func(&db)
}

#[allow(clippy::iter_skip_zero)]
fn remove_dupes_old(c: &mut Criterion) {
    let amount: usize = black_box(1_000);
    let dedupe_amount: i32 = black_box(100);
    c.bench_function("dupes", |b| {
        b.iter(|| {
            create_and_fill_db(amount, |db| {
                let read_tx = db.begin_read()?;
                let write_tx = db.begin_write()?;

                {
                    let read_table = read_tx.open_table(TABLE_DEF)?;
                    let mut table = write_tx.open_table(TABLE_DEF)?;

                    let cursor = read_table.iter()?;
                    let mut seen = HashSet::<Vec<u8>>::new();
                    let len = table.len()? as usize;

                    match dedupe_amount.cmp(&0) {
                        Greater => Left(cursor.rev().skip(len - dedupe_amount as usize)),
                        Less => Right(cursor.skip(dedupe_amount.unsigned_abs() as usize)),
                        Equal => Left(cursor.rev().skip(0)),
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
            })
        })
    });
}

#[allow(clippy::iter_skip_zero)]
fn remove_dupes_iter(c: &mut Criterion) {
    let amount: usize = black_box(1_000);
    let dedupe_amount: i32 = black_box(100);
    c.bench_function("dupes", |b| {
        b.iter(|| {
            create_and_fill_db(amount, |db| {
                let mut seen = HashSet::<Vec<u8>>::new();

                let write_tx = db.begin_write()?;
                {
                    let mut table = write_tx.open_table(TABLE_DEF)?;
                    let cursor = db.begin_read()?.open_table(TABLE_DEF)?;

                    match dedupe_amount.cmp(&0) {
                        Greater => Left(
                            cursor
                                .iter()?
                                .rev()
                                .skip(table.len()? as usize - dedupe_amount as usize),
                        ),
                        Less => Right(cursor.iter()?.skip(dedupe_amount.unsigned_abs() as usize)),
                        Equal => Left(cursor.iter()?.rev().skip(0)),
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
            })
        })
    });
}

criterion_group!(benches, remove_dupes_old, remove_dupes_iter);
criterion_main!(benches);

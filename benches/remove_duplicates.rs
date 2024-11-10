use chrono::Local;
use clippy::utils::database::{remove_duplicates, TABLE_DEF};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{distributions::Alphanumeric, Rng};
use redb::Database;
use scopeguard::defer;
use std::{fs, hint::black_box, ops::FnOnce};
use tempfile::NamedTempFile;

pub fn get_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}

fn create_and_fill_db<F>(amount: usize, func: F)
where
    F: FnOnce(&Database),
{
    let dupe = "asdf";
    let tmp = NamedTempFile::new().unwrap().into_temp_path();
    let path = tmp.to_str().unwrap().to_string();
    tmp.close().unwrap();

    let db = Database::create(&path).unwrap();
    defer!(fs::remove_file(path).unwrap());

    for i in 0..amount {
        let dummy = match i % 10 {
            0 => get_random_string().into_bytes(),
            _ => dupe.to_string().into_bytes(),
        };

        let tx = db.begin_write().unwrap();
        {
            let mut table = tx.open_table(TABLE_DEF).unwrap();
            table
                .insert(Local::now().timestamp_micros(), dummy.to_vec())
                .unwrap();
        }
        tx.commit().unwrap();
    }
    func(&db);
}

fn remove_dupes_custom(size: usize, dedupe_amount: i32) {
    create_and_fill_db(size, |db| {
        remove_duplicates(db, dedupe_amount).unwrap();
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("dupes 20", |b| {
        b.iter(|| remove_dupes_custom(black_box(100), black_box(20)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


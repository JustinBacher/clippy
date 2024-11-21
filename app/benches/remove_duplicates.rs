use std::hint::black_box;

use anyhow::Result;
use camino::Utf8Path;
use clippy::database::{get_db, remove_duplicates, ClipEntry, Database};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{distributions::Alphanumeric, Rng};
use shortcut_assert_fs::TmpFs;

pub fn get_random_string() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect()
}

fn create_and_fill_db<F>(amount: i64, func: F) -> Result<()>
where
    F: FnOnce(&Database) -> Result<()>,
{
    let tf = TmpFs::new()?;
    let path = tf.path("test");
    let db = get_db(Utf8Path::new(path.as_str()))?;
    let tx = db.rw_transaction()?;

    {
        for i in 0..amount {
            let payload = match i % 10 {
                0 => &get_random_string(),
                _ => "asdf",
            };
            tx.insert(ClipEntry::new(payload.as_bytes()))?;
        }
    }
    tx.commit()?;

    func(&db)
}

#[allow(clippy::iter_skip_zero)]
fn remove_dupes_old(c: &mut Criterion) {
    let amount: i64 = black_box(1_000);
    let dedupe_amount: i64 = black_box(100);
    c.bench_function("dupes", |b| {
        b.iter(|| {
            create_and_fill_db(amount, |db| {
                remove_duplicates(db, dedupe_amount)?;
                Ok(())
            })
        })
    });
}

#[allow(clippy::iter_skip_zero)]
fn remove_dupes_iter(c: &mut Criterion) {
    let amount: i64 = black_box(1_000);
    let dedupe_amount: i64 = black_box(100);
    c.bench_function("dupes", |b| {
        b.iter(|| {
            create_and_fill_db(amount, |db| {
                remove_duplicates(db, dedupe_amount)?;
                Ok(())
            })
        })
    });
}

criterion_group!(benches, remove_dupes_old, remove_dupes_iter);
criterion_main!(benches);

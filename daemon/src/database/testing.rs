extern crate shortcut_assert_fs;
use itertools::Itertools;
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
            tx.insert(ClipEntry::new(dummy.as_bytes()))?;
        }
        tx.commit()?;

        all_items.push(dummy.as_bytes().to_vec());
    }

    func(&db, all_items)
}

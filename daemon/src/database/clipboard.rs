use std::collections::HashSet;

use anyhow::Result;
use camino::Utf8Path;
pub use native_db::{
    native_db,
    transaction::{RTransaction, RwTransaction},
    Builder as DatabaseBuilder, Database, Key, Models, ToInput, ToKey,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;
pub use schemas::ClipEntry;

pub mod schemas {
    use super::*;
    #[cfg(target_os = "linux")]
    use crate::utils::get_focused_window;
    use anyhow::Result;
    use native_model::{native_model, Model};

    pub type ClipEntry = v1::ClipEntryV1;

    mod v1 {
        use super::*;

        #[native_db]
        #[native_model(id = 1, version = 1, with = Bincode)]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
        pub struct ClipEntryV1 {
            #[primary_key]
            pub id: u128,
            #[secondary_key]
            pub epoch: DateTime,
            pub payload: Vec<u8>,
            pub application: Option<String>,
        }

        impl ClipEntryV1 {
            pub fn new(payload: &[u8]) -> Self {
                Self {
                    id: Uuid::now_v7().to_u128_le(),
                    epoch: v1::DateTime::now(),
                    payload: payload.to_vec(),
                    application: get_focused_window(),
                }
            }

            pub fn text(&self) -> Result<String> {
                Ok(std::str::from_utf8(&self.payload)?.to_string())
            }

            pub fn contains(&self, query: &Option<String>) -> bool {
                if let Some(check) = query {
                    return self.text().is_ok_and(|text| text.contains(check));
                }
                false
            }

            pub fn was_copied_from_app(&self, maybe_title: &Option<String>) -> bool {
                if let Some(title) = maybe_title {
                    if self.application.as_deref().unwrap_or_default().contains(title) {
                        return true;
                    }
                }

                false
            }
        }
    }
}

pub static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<ClipEntry>().unwrap();
    models
});

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

pub fn get_db(path: &Utf8Path) -> Result<native_db::Database> {
    let db = DatabaseBuilder::new().create(&MODELS, path)?;
    let tx = db.rw_transaction()?;

    tx.migrate::<ClipEntry>()?;
    tx.commit()?;

    Ok(db)
}

pub fn remove_duplicates(
    db: &Database,
    to_remove: &Option<u64>,
    to_keep: &Option<u64>,
) -> Result<()> {
    let rtx = db.r_transaction()?;
    let wtx = db.rw_transaction()?;
    let cursor = rtx.scan().primary::<ClipEntry>()?;
    let seen = &mut HashSet::<Vec<u8>>::new();
    let amount: usize;

    // I'm using Box dyn to save myself a bunch of repetitive if statements
    let mut filtered: Box<dyn Iterator<Item = ClipEntry>> = {
        if let Some(remove_amount) = to_remove {
            amount = *remove_amount as usize;
            Box::new(cursor.all()?.flatten())
        } else if let Some(keep_amount) = to_keep {
            amount = *keep_amount as usize;
            Box::new(cursor.all()?.rev().flatten())
        } else {
            amount = 0;
            Box::new(cursor.all()?.rev().flatten())
        }
    };

    filtered = Box::new(filtered.filter(|entry| !seen.insert(entry.payload.to_vec())));

    if amount != 0 {
        filtered = Box::new(filtered.skip(amount));
    }

    filtered.for_each(|entry| {
        wtx.remove(entry).ok();
    });

    Ok(wtx.commit()?)
}

pub fn ensure_db_size(db: &Database, limit: &u64) -> Result<()> {
    let tx = db.rw_transaction()?;
    let len = tx.length()?;

    if *limit > len {
        return Ok(());
    }

    let amount = limit - len;

    tx.scan()
        .primary::<ClipEntry>()?
        .all()?
        .take(amount as usize)
        .flatten()
        .for_each(|entry| {
            tx.remove(entry).unwrap();
        });

    Ok(tx.commit()?)
}

#[cfg(test)]
pub mod test {

    use crate::database::testing::{fill_db_and_test, FillWith};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_removes_dupes_oldest() {
        let dupe = "asdf";
        fill_db_and_test(FillWith::DupesRandomEnds(dupe), 20, |db, _| {
            remove_duplicates(db, &Some(10), &None)?;
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
            remove_duplicates(db, &None, &Some(10))?;
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
            remove_duplicates(db, &Some(0), &None)?;
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
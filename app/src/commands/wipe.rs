use std::cell::RefCell;

use clap::Parser;
use redb::{Database, ReadableTable};

use super::ClippyCommand;
use crate::{cli::ClippyCli, prelude::Result, utils::database::TABLE_DEF};

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Wipe {}

impl ClippyCommand for Wipe {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let db = Database::open(&args.db_path)?;
        let tx = db.begin_write()?;
        {
            let table = RefCell::new(tx.open_table(TABLE_DEF)?);
            let read_table = table.borrow();
            let cursor = read_table.iter()?;

            cursor.for_each(|entry| {
                table.borrow_mut().remove(entry.as_ref().unwrap().0.value()).unwrap();
            });
        }
        tx.commit()?;

        Ok(())
    }
}
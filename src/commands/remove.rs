use super::recall::GreedyInt;
use super::ClippyCommand;
use crate::{cli::Cli, prelude::Result, utils::database::TABLE_DEF};
use clap::Parser;
use derive_more::From;
use redb::{Database, ReadableTable};
use std::cell::RefCell;

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub(crate) struct Remove {
    /// The id of the clip from the output of `list` command
    id: GreedyInt,
}

impl ClippyCommand for Remove {
    fn execute(&self, args: &Cli) -> Result<()> {
        let db = Database::open(&args.db_path)?;
        let tx = db.begin_write()?;
        {
            let table = RefCell::new(tx.open_table(TABLE_DEF)?);
            let read_table = table.borrow();
            let cursor = read_table.iter()?;

            cursor.skip(self.id.into()).for_each(|entry| {
                table
                    .borrow_mut()
                    .remove(entry.as_ref().unwrap().0.value())
                    .unwrap();
            });
        }
        tx.commit()?;

        Ok(())
    }
}

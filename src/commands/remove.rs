use super::ClippyCommand;
use super::GreedyInt;
use crate::{cli::App, prelude::Result, utils::database::TABLE_DEF};
use clap::Parser;
use derive_more::From;
use redb::{Database, ReadableTable, ReadableTableMetadata};

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub struct Remove {
    /// The id of the clip from the output of `list` command
    id: GreedyInt,
}

impl ClippyCommand for Remove {
    fn execute(&self, args: &App) -> Result<()> {
        let position: usize = self.id.into();
        let db = Database::open(&args.db_path)?;
        let read_tx = db.begin_read()?;
        let read_table = read_tx.open_table(TABLE_DEF)?;

        {
            if read_table.len()? == 0 {
                println!("Clipboard empty. There is nothing to remove.");
                return Ok(());
            }
        }

        let write_tx = db.begin_write()?;

        {
            let mut cursor = read_table.iter()?;
            let mut write_table = write_tx.open_table(TABLE_DEF)?;

            write_table.remove(cursor.nth(position - 1).unwrap()?.0.value())?;
        }
        write_tx.commit()?;

        Ok(())
    }
}

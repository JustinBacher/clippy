use super::ClippyCommand;
use super::GreedyInt;
use crate::{cli::Cli, prelude::Result, utils::database::TABLE_DEF};
use clap::Parser;
use derive_more::From;
use redb::{Database, ReadableTable, ReadableTableMetadata};

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub(crate) struct Remove {
    /// The id of the clip from the output of `list` command
    id: GreedyInt,
}

impl ClippyCommand for Remove {
    fn execute(&self, args: &Cli) -> Result<()> {
        let position: usize = self.id.into();
        let db = Database::open(&args.db_path)?;
        let read_tx = db.begin_read()?;
        let read_table = read_tx.open_table(TABLE_DEF)?;

        {
            if read_table.len()? == 0 {
                println!("Clipboard empty. There is nothing to remove.");
                ()
            }
        }

        let write_tx = db.begin_write()?;

        {
            let cursor = Box::new(read_table.iter()?);

            write_tx
                .open_table(TABLE_DEF)?
                .remove(cursor.skip(position - 1).next().unwrap()?.0.value())?;
        }
        write_tx.commit()?;

        Ok(())
    }
}

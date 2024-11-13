use clap::Parser;
use redb::{Database, ReadableTable, ReadableTableMetadata};

use super::{ClippyCommand, GreedyInt};
use crate::{cli::ClippyCli, prelude::Result, utils::database::TABLE_DEF};

#[derive(Parser, Debug, PartialEq)]
#[command(allow_missing_positional(true))]
/// Outputs clip to `stdout`.
///
/// Meant for use with `wl-paste`
pub struct Recall {
    /// The id of the clip to use.
    ///
    /// From the output of `list` command
    id: GreedyInt,
    #[arg(hide = true)] // This is just to make clap stop complaining
    other: Option<Vec<String>>,
}

impl ClippyCommand for Recall {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;

        {
            let table = tx.open_table(TABLE_DEF)?;

            if table.is_empty()? {
                println!("There is no clip with that id");
                return Ok(());
            }

            let clip = table.iter()?.nth(&self.id - 1).unwrap()?.1.value();
            println!("{}", std::str::from_utf8(clip.as_slice()).unwrap());
        }

        Ok(())
    }
}

use super::{ClippyCommand, GreedyInt};
use crate::{cli::Cli, prelude::Result, utils::database::TABLE_DEF};
use clap::Parser;
use redb::{Database, ReadableTable};
use std::io::{stdout, Write};

#[derive(Parser, Debug, PartialEq)]
#[command(allow_missing_positional(true))]
/// Outputs clip to `stdout`.
///
/// Meant for use with `wl-paste`
pub(crate) struct Recall {
    #[arg()]
    /// The id of the clip to use.
    ///
    /// From the output of `list` command
    id: GreedyInt,
    #[arg()]
    other: Option<Vec<String>>,
}

impl ClippyCommand for Recall {
    fn execute(&self, args: &Cli) -> Result<()> {
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;
        {
            let table = tx.open_table(TABLE_DEF)?;
            let mut out = stdout();

            out.write_all(&table.iter()?.skip(&self.id - 1).next().unwrap()?.1.value())?;
        }
        Ok(())
    }
}

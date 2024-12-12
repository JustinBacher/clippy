use anyhow::Result;
use clap::Parser;
use clippy_daemon::database::clipboard::{get_db, ClipEntry, TableLen};

use super::{ClippyCommand, GreedyInt};
use crate::cli::ClippyCli;

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
        let error_text = "There is no clip with that id";
        let db = get_db(&args.db_path)?;
        let tx = db.r_transaction()?;

        if tx.length()? == 0 {
            println!("{error_text}");
            return Ok(());
        }

        let clip = tx
            .scan()
            .primary::<ClipEntry>()?
            .all()?
            .flatten()
            .nth(&self.id - 1)
            .expect(error_text)
            .text()?;
        println!("{clip}");

        Ok(())
    }
}

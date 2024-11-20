use anyhow::Result;
use clap::Parser;

use super::{ClippyCommand, GreedyInt};
use crate::{
    cli::ClippyCli,
    database::{get_db, ClipEntry, EasyLength, PrimaryScanIterator},
};

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
        let db = get_db(args)?;
        let tx = db.r_transaction()?;

        if tx.length()? == 0 {
            println!("{error_text}");
            return Ok(());
        }

        let it = tx.scan().primary()?;
        let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;

        let clip = cursor.flatten().nth(&self.id - 1).expect(error_text).payload;
        println!("{}", clip);

        Ok(())
    }
}

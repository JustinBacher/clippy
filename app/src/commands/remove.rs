use anyhow::Result;
use clap::Parser;
use derive_more::From;

use super::{ClippyCommand, GreedyInt};
use crate::{
    cli::ClippyCli,
    database::{get_db, ClipEntry, EasyLength, PrimaryScanIterator},
};

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub struct Remove {
    /// The id of the clip from the output of `list` command
    id: GreedyInt,
}

impl ClippyCommand for Remove {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let position: usize = self.id.into();
        let db = get_db(args)?;
        let tx = db.rw_transaction()?;

        {
            if tx.length()? == 0 {
                println!("Clipboard empty. There is nothing to remove.");
                return Ok(());
            }
        }

        {
            let it = tx.scan().primary()?;
            let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
            tx.remove(cursor.flatten().nth(position - 1).expect("No clip found at that index"))?;
        }
        tx.commit()?;

        Ok(())
    }
}

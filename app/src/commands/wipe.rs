use anyhow::Result;
use clap::Parser;

use super::ClippyCommand;
use crate::{cli::ClippyCli, database::get_db};

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Wipe {}

impl ClippyCommand for Wipe {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let db = get_db(args)?;
        let tx = db.rw_transaction()?;
        tx.drain();
        tx.commit()?;

        Ok(())
    }
}

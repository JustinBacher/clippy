use anyhow::Result;
use clap::Parser;
use clippy_daemon::database::get_db;

use super::ClippyCommand;
use crate::cli::ClippyCli;

#[derive(Parser, Debug, PartialEq)]
/// Wipes all clips from clipboard
pub struct Wipe {}

impl ClippyCommand for Wipe {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let db = get_db(&args.db_path)?;
        let tx = db.rw_transaction()?;
        tx.drain();
        tx.commit()?;

        Ok(())
    }
}

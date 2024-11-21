use std::io::{stdout, Write};

use anyhow::Result;
use clap::Parser;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    database::{get_db, ClipEntry, EasyLength, PrimaryScanIterator},
    utils::formatting::format_entry,
};

#[derive(Parser, Debug, PartialEq)]
/// Searches for a clip that contains `query`
pub struct Search {
    #[arg(short, long)]
    /// The query to search for in clipboard history
    query: String,

    #[arg(short('d'), long, action)]
    /// Includes dates clips were taken in the output
    include_dates: bool,

    #[arg(short('w'), long, default_value = "100")]
    /// Max characters to show of clips in preview. Use 0 to retain original width.
    ///
    ///
    /// This does not affect what is put back into the clipboard
    preview_width: usize,
}

impl ClippyCommand for Search {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let mut out = stdout();
        let db = get_db(&args.db_path)?;
        let tx = db.r_transaction()?;
        {
            let count = tx.length()? as usize;

            let it = tx.scan().primary()?;
            let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
            cursor
                .flatten()
                .enumerate()
                .map(|(i, entry)| (i, format_entry(&entry, self.preview_width)))
                .filter(|(_, entry)| entry.1.contains(&self.query))
                .for_each(|(i, entry)| {
                    let (date, payload) = entry;
                    match self.include_dates {
                        true => writeln!(out, "{}. {}:\t{}", count - i, date, payload).unwrap(),
                        false => writeln!(out, "{}. {}\n", count - i, payload).unwrap(),
                    }
                });
        }
        Ok(())
    }
}

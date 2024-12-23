use std::io::{stdout, Write};

use anyhow::Result;
use clap::Parser;
use clippy_daemon::database::{get_db, ClipEntry, TableLen};

use super::ClippyCommand;
use crate::{cli::ClippyCli, utils::formatting::format_entry};

#[derive(Parser, Debug, PartialEq)]
/// Searches for a clip that contains `query`
pub struct Search {
    #[arg(short, long)]
    /// The query to search for in clipboard history
    query: Option<String>,

    #[arg(short, long, visible_alias("app"))]
    /// Filter search results to clips from a specific application.
    application: Option<String>,

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

        if tx.length()? == 0 {
            println!("Clipboard is empty");
            return Ok(());
        }

        tx.scan()
            .primary::<ClipEntry>()?
            .all()?
            .flatten()
            .enumerate()
            .filter(|(_, entry)| {
                entry.contains(&self.query) & entry.was_copied_from_app(&self.application)
            })
            .for_each(|(i, entry)| {
                let preview = format_entry(&entry, self.preview_width, self.include_dates);
                writeln!(out, "{i} {}", preview,).unwrap();
            });
        Ok(())
    }
}

use super::ClippyCommand;
use crate::{
    cli::App,
    prelude::Result,
    utils::{database::TABLE_DEF, formatting::format_entry},
};
use clap::Parser;
use redb::{Database, ReadableTable, ReadableTableMetadata};
use std::io::{stdout, Write};

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
    /// This does not affect what is put back into the clipboard
    preview_width: usize,
}

impl ClippyCommand for Search {
    fn execute(&self, args: &App) -> Result<()> {
        let mut out = stdout();
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;
        {
            let table = tx.open_table(TABLE_DEF)?;
            let count = table.len().unwrap() as usize;

            table
                .iter()?
                .enumerate()
                .map(|(i, entry)| (i, format_entry(entry.unwrap(), self.preview_width)))
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

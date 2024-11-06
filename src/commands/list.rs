use super::ClippyCommand;
use crate::{
    cli::Cli,
    prelude::Result,
    utils::{database::TABLE_DEF, formatting::format_entry},
};
use clap::{Parser, ValueEnum};
use redb::{Database, ReadableTable};
use serde::Serialize;
use std::io::{stdout, Write};

const FIVE_MEGABYTES: usize = 5e6 as usize;

#[derive(ValueEnum, Parser, Clone, Default, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardState {
    #[default]
    Nil,
    Data,
    Clear,
    Sensitive,
}

#[derive(Parser, Debug, PartialEq)]
/// Lists all stored clips in clipboard
pub(crate) struct List {
    #[arg(short('d'), long, action)]
    /// Includes dates clips were taken in the output
    include_dates: bool,

    #[arg(short('w'), long, default_value = "100")]
    /// Max characters to show of clips in preview. Use 0 to retain original width.
    ///
    /// This does not affect what is put back into the clipboard
    preview_width: usize,
}

impl ClippyCommand for List {
    fn execute(&self, args: &Cli) -> Result<()> {
        let mut out = stdout();
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;
        {
            let table = tx.open_table(TABLE_DEF)?;
            let count = table.iter()?.count();

            table.iter()?.enumerate().for_each(|(i, entry)| {
                let (date, payload) = format_entry(entry.unwrap(), self.preview_width);

                match self.include_dates {
                    true => out.write_fmt(format_args!("{}. {}:\t{}\n", count - i, date, payload)),
                    false => out.write_fmt(format_args!("{}. {}\n", count - i, payload)),
                }
                .unwrap();
            });
        }

        Ok(())
    }
}

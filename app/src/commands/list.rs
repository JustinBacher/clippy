use std::io::{stdout, Write};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Serialize;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    database::{get_db, ClipEntry, TableLen},
    utils::formatting::format_entry,
};

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
pub struct List {
    /// Includes dates clips were taken in the output
    #[arg(short('d'), long, action, default_value = "false")]
    include_dates: bool,

    /// Max characters to show of clips in preview. Use 0 to retain original width.
    ///
    /// This does not affect what is put back into the clipboard
    #[arg(short('w'), long, default_value = "100")]
    preview_width: usize,
}

impl ClippyCommand for List {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        let mut out = stdout();
        let db = get_db(&args.db_path)?;
        let tx = db.r_transaction()?;

        if tx.length()? == 0 {
            return Ok(println!("Clipboard is empty"));
        }

        tx.scan()
            .primary::<ClipEntry>()?
            .all()?
            .flatten()
            .enumerate()
            .for_each(|(i, entry)| {
                let preview = format_entry(&entry, self.preview_width, self.include_dates);
                writeln!(out, "{i} {}", preview,).unwrap();
            });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cli::mock_cli,
        database::test::{fill_db_and_test, get_db_contents, FillWith},
    };

    #[test]
    fn it_lists() {
        fill_db_and_test(FillWith::Random, 20, |db, before| {
            mock_cli(std::iter::once("list"));

            let after = get_db_contents(db)?;

            Ok(assert_eq!(after, before))
        })
        .unwrap();
    }
}

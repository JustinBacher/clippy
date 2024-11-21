use std::io::{stdout, Write};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Serialize;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    database::{get_db, ClipEntry, EasyLength, PrimaryScanIterator},
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
        {
            let count = tx.length()? as usize;

            if count == 0 {
                println!("Clipboard is empty. Ready for you to start copying");
                return Ok(());
            }

            let it = tx.scan().primary()?;
            let cursor: PrimaryScanIterator<ClipEntry> = it.all()?;
            cursor.flatten().enumerate().for_each(|(i, entry)| {
                let (date, payload) = format_entry(&entry, self.preview_width);

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

            assert_eq!(after, before);

            Ok(())
        })
        .unwrap();
    }
}

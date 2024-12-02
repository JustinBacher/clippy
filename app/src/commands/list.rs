use std::path::Path;

use anyhow::Result;
use camino::Utf8Path;
use clap::Parser;
use clippy_daemon::{
    database::{ClipEntry, TableLen, get_db},
    utils::config::Config,
};
use futures::executor;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    utils::{formatting::format_entry, get_config_path},
};

#[derive(Parser, Debug, PartialEq)]
/// Lists all stored clips in clipboard
pub struct List {
    /// What clipboard to list clips from
    #[arg(short, long, default_value = "primary")]
    clipboard: Option<String>,

    /// Includes dates clips were taken in the output
    #[arg(short('d'), long, action, default_value = "false")]
    include_dates: bool,

    /// Max characters to show of clips in preview. Use 0 to retain original
    /// width.
    ///
    /// This does not affect what is put back into the clipboard
    #[arg(short('w'), long, default_value = "100")]
    preview_width: usize,
}

impl ClippyCommand for List {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        let config =
            executor::block_on(Config::from_file(Path::new(&get_config_path("clippy", "config.toml").unwrap())))?;
        let mut clipboard = self.clipboard.clone().unwrap();
        if clipboard == "primary" {
            clipboard = "default".to_string();
        }
        let Some(board) = config.clipboard.get(&clipboard) else {
            print!("No clipboards with the name: {}", self.clipboard.as_ref().unwrap());
            return Ok(());
        };
        let db = get_db(Utf8Path::new(&board.db_path))?;
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
            .for_each(|(i, entry)| {
                let preview = format_entry(&entry, self.preview_width, self.include_dates);
                println!("{i} {}", preview);
            });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use clippy_daemon::database::testing::{FillWith, fill_db_and_test, get_db_contents};

    use crate::cli::mock_cli;

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

use super::ClippyCommand;
use crate::{
    cli::App,
    prelude::Result,
    utils::{
        database::{remove_duplicates, TABLE_DEF},
        formatting::trim,
    },
};
use chrono::Local;
use clap::{ArgAction, Parser, ValueEnum};
use redb::Database;

use serde::Serialize;
use std::{
    io::{stdin, Read},
    mem::size_of_val,
};

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
/// Reads a clip from stdin and remembers it for later recall
pub struct Store {
    #[arg(env, action=ArgAction::Set, hide(true))]
    clipboard_state: ClipboardState,
}

impl ClippyCommand for Store {
    fn execute(&self, args: &App) -> Result<()> {
        match self.clipboard_state {
            ClipboardState::Sensitive => todo!("Use non-persistent storage for secrets"),
            ClipboardState::Clear | ClipboardState::Nil => {
                println!("Should be warning");
                warn!(
                    "
                    Clippy does not implement \"nil\" or \"clear\" for `CLIPBOARD_STATE`
                    Environment Variable.
                    Please use clippy with
                    [wl-clipboard](https://github.com/bugaevc/wl-clipboard)"
                );
            }
            ClipboardState::Data => {
                let db = Database::open(&args.db_path)?;
                let mut payload = Vec::new();
                stdin().read_to_end(&mut payload)?;
                store(&db, payload)?;
                remove_duplicates(&db, args.duplicates)?;
            }
        }
        Ok(())
    }
}

pub fn store(db: &Database, payload: Vec<u8>) -> Result<Vec<u8>> {
    if size_of_val(&payload) > FIVE_MEGABYTES || trim(&payload).is_empty() {
        panic!("Data too large")
    }

    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE_DEF)?;
        table.insert(Local::now().timestamp_micros(), payload.to_owned())?;
    }
    tx.commit()?;
    Ok(payload)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::database::test::{fill_db_and_test, FillWith};
    use redb::ReadableTableMetadata;

    #[test]
    fn it_stores() {
        fill_db_and_test(FillWith::Random, |db, _| {
            let count = db.begin_read()?.open_table(TABLE_DEF)?.len()?;

            Ok(assert_eq!(count, 20))
        })
        .unwrap();
    }
}

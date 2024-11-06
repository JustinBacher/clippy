use super::ClippyCommand;
use crate::{
    cli::Cli,
    prelude::Result,
    utils::{
        database::{remove_duplicates, TABLE_DEF},
        formatting::trim,
    },
};
use camino::Utf8PathBuf;
use chrono::Local;
use clap::{ArgAction, Parser, ValueEnum};
use redb::Database;
use serde::Serialize;
use std::io::{stdin, Read, Stdin};

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
pub(crate) struct Store {
    #[arg(env("CLIPBOARD_STATE"), action=ArgAction::Set, hide(true))]
    clipboard_state: ClipboardState,
}

impl ClippyCommand for Store {
    fn execute(&self, args: Cli) -> Result<()> {
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
                let db_path = args.db_path;
                store(&db_path, &mut stdin())?;
                remove_duplicates(&db_path, args.duplicates, args.keep)?;
            }
        }
        Ok(())
    }
}

fn store(db_path: &Utf8PathBuf, input: &mut Stdin) -> Result<()> {
    let mut payload = Vec::new();
    input.read_to_end(&mut payload)?;

    if std::mem::size_of_val(&payload) > FIVE_MEGABYTES || trim(&payload).len() == 0 {
        ()
    }

    let db = Database::create(&db_path)?;
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE_DEF)?;
        table.insert(Local::now().timestamp_millis(), payload.to_owned())?;
    }
    tx.commit()?;

    Ok(())
}

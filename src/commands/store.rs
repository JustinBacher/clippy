use super::ClippyCommand;
use crate::{
    cli::App,
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
pub(crate) struct Store {
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
                let db_path = &args.db_path;
                let mut payload = Vec::new();
                stdin().read_to_end(&mut payload)?;
                store(db_path, payload)?;
                remove_duplicates(db_path, args.duplicates, args.keep)?;
            }
        }
        Ok(())
    }
}

fn store(db_path: &Utf8PathBuf, payload: Vec<u8>) -> Result<()> {
    if size_of_val(&payload) > FIVE_MEGABYTES || trim(&payload).is_empty() {
        return Ok(());
    }

    let db = Database::create(db_path)?;
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE_DEF)?;
        table.insert(Local::now().timestamp_millis(), payload.to_owned())?;
    }
    tx.commit()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use fake::{Fake, StringFaker};
    use redb::ReadableTableMetadata;
    use tempfile::NamedTempFile;

    const ASCII: &str =
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&\'()*+,-./:;<=>?@";

    #[test]
    fn it_stores() {
        let faker = StringFaker::charset(ASCII.into());
        let tmp = NamedTempFile::new().unwrap().into_temp_path();

        let path = Utf8PathBuf::from("/tmp/db");
        // let path = Utf8PathBuf::from(tmp.to_str().unwrap().to_string());
        println!("{}", path);
        tmp.close().unwrap();

        for i in 0..100 {
            let payload = match i % 10 {
                0 => "asdf".to_string(),
                _ => faker.fake(),
            };

            println!("Testing storing: {}", payload);
            store(&path, payload.into_bytes()).unwrap();
        }

        let count = Database::create(path)
            .unwrap()
            .begin_read()
            .unwrap()
            .open_table(TABLE_DEF)
            .unwrap()
            .len()
            .unwrap();

        assert_eq!(count, 10);
    }
}

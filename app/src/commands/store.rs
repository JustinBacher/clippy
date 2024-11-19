use std::{
    io::{stdin, Read},
    mem::size_of_val,
};

use chrono::Local;
use clap::{ArgAction, Parser, ValueEnum};
use redb::Database;
use serde::Serialize;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    prelude::Result,
    utils::{
        database::{ensure_db_size, remove_duplicates, TABLE_DEF},
        formatting::trim,
    },
};

const FIVE_MEGABYTES: usize = 5e6 as usize;

#[derive(ValueEnum, Parser, Clone, Default, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    #[default]
    Other,
    Nil,
    Data,
    Clear,
    Sensitive,
}

#[derive(Parser, Debug, PartialEq)]
/// Reads a clip from stdin and remembers it for later recall
pub struct Store {
    #[arg(env, action=ArgAction::Set, hide(true))]
    clipboard_state: State,
}

impl ClippyCommand for Store {
    fn execute(&self, args: &ClippyCli) -> Result<()> {
        match self.clipboard_state {
            State::Data => {
                let db = Database::open(&args.db_path)?;
                let mut payload = Vec::new();
                stdin().read_to_end(&mut payload)?;
                store(&db, payload)?;
                remove_duplicates(&db, args.duplicates)?;
                ensure_db_size(&db, args.keep)?;
            },
            State::Sensitive => todo!("Use non-persistent storage for secrets"),
            State::Clear | State::Nil => (), // May want to implement these at some point
            State::Other => (),
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
    use redb::ReadableTableMetadata;

    use super::*;
    use crate::utils::database::test::{fill_db_and_test, get_db_contents, FillWith};

    #[test]
    fn it_stores() {
        fill_db_and_test(FillWith::Random, 20, |db, before| {
            let count = db.begin_read()?.open_table(TABLE_DEF)?.len()?;

            assert_eq!(count, 20);
            assert_eq!(get_db_contents(db)?, before);

            Ok(())
        })
        .unwrap();
    }
}

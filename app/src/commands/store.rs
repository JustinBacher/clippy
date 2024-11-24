use std::{
    io::{stdin, Read},
    mem::size_of_val,
};

use anyhow::Result;
use clap::{ArgAction, Parser, ValueEnum};
use serde::Serialize;

use super::ClippyCommand;
use crate::{
    cli::ClippyCli,
    database::{ensure_db_size, get_db, remove_duplicates, ClipEntry, Database},
    utils::formatting::trim,
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
                let db = get_db(&args.db_path)?;
                let mut payload = Vec::new();
                stdin().read_to_end(&mut payload)?;
                store(&db, payload.as_slice())?;
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

pub fn store(db: &Database, payload: &[u8]) -> Result<()> {
    if size_of_val(payload) > FIVE_MEGABYTES || trim(payload).is_empty() {
        panic!("Data too large")
    }

    let tx = db.rw_transaction()?;
    {
        tx.insert(ClipEntry::new(payload))?;
    }
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod test {

    use crate::database::{
        test::{fill_db_and_test, get_db_contents, FillWith},
        TableLen,
    };

    #[test]
    fn it_stores() {
        fill_db_and_test(FillWith::Random, 20, |db, before| {
            let count = db.r_transaction()?.length()?;

            assert_eq!(count, 20);
            assert_eq!(get_db_contents(db)?, before);

            Ok(())
        })
        .unwrap();
    }
}

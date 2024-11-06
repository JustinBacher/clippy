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

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub(crate) struct Store {
    /// The id of the clip from the output of list command
    id: Option<usize>,
}

impl ClippyCommand for Store {
    fn execute(&self, args: Cli) -> Result<()> {
        todo!()
    }
}

#[derive(ValueEnum, Parser, Clone, Default, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardState {
    #[default]
    Nil,
    Data,
    Clear,
    Sensitive,
}

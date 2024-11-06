use super::ClippyCommand;
use crate::{cli::Cli, prelude::Result, utils::database::TABLE_DEF};
use clap::Parser;
use derive_more::Display;
use redb::{Database, ReadableTable};
use std::{
    default, io::{stdout, Error as IOError, Write}, ops::Sub, str::FromStr
};

#[derive(Parser, Debug, PartialEq)]
#[command(allow_missing_positional(true))]
/// Outputs clip to `stdout`.
///
/// Meant for use with `wl-paste`
pub(crate) struct Recall {
    #[arg()]
    /// The id of the clip to use.
    ///
    /// From the output of `list` command
    id: GreedyInt,
    #[arg()]
    other: Option<Vec<String>>,
}

impl ClippyCommand for Recall {
    fn execute(&self, args: &Cli) -> Result<()> {
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;
        {
            let table = tx.open_table(TABLE_DEF)?;
            let mut out = stdout();

            out.write_all(&table.iter()?.skip(&self.id - 1).next().unwrap()?.1.value())?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, Display)]
pub struct GreedyIntParseError;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GreedyInt {
    data: usize,
}

impl FromStr for GreedyInt {
    type Err = std::io::Error;


    fn from_str(s: &str) -> Result<GreedyInt> {
        Ok(s.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .map(|x| GreedyInt{data: x})
            .unwrap())
    }
}

impl Into<usize> for GreedyInt {
    fn into(self) -> usize {
        self.data
    }
}

impl Into<GreedyInt> for usize {
    fn into(self) -> GreedyInt {
        GreedyInt { data: self }
    }
}

impl Sub<usize> for &GreedyInt {
    type Output = usize;

    fn sub(self, other: usize) -> usize {
        self.data - other
    }
}

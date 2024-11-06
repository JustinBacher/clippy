use super::ClippyCommand;
use crate::{cli::Cli, prelude, utils::database::TABLE_DEF};
use clap::Parser;
use derive_more::{Display, From, Sub};
use redb::{Database, ReadableTable};
use std::{
    io::{stdout, Write},
    str::FromStr,
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
    fn execute(&self, args: Cli) -> prelude::Result<()> {
        let db = Database::create(&args.db_path)?;
        let tx = db.begin_read()?;
        {
            let table = tx.open_table(TABLE_DEF)?;
            let mut out = stdout();

            out.write_all(
                &table
                    .iter()?
                    .skip(self.id.into() - 1)
                    .next()
                    .unwrap()?
                    .1
                    .value(),
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error, Display)]
pub struct GreedyIntParseError;

#[derive(Clone, PartialEq, Eq, Debug, From, Sub)]
pub struct GreedyInt(usize);

impl FromStr for GreedyInt {
    type Err = GreedyIntParseError;

    fn from_str(s: &str) -> Result<GreedyInt, GreedyIntParseError> {
        Ok(s.chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| GreedyIntParseError)?
            .into())
    }
}

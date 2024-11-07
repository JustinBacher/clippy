pub mod completions;
pub mod list;
pub mod recall;
pub mod remove;
pub mod search;
pub mod store;
pub mod version;
pub mod wipe;

pub(crate) use completions::GenCompletions;
pub(crate) use list::List;
pub(crate) use recall::Recall;
pub(crate) use remove::Remove;
pub(crate) use search::Search;
pub(crate) use store::Store;
pub(crate) use version::Version;
pub(crate) use wipe::Wipe;

use crate::cli::Cli;
use crate::prelude::{Error, Result};
use std::{ops::Sub, str::FromStr};

pub trait ClippyCommand {
    #[allow(dead_code)]
    fn execute(&self, args: &Cli) -> Result<()>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GreedyInt {
    data: usize,
}

impl FromStr for GreedyInt {
    type Err = Error;

    fn from_str(s: &str) -> Result<GreedyInt> {
        Ok(GreedyInt {
            data: s
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .unwrap(),
        })
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

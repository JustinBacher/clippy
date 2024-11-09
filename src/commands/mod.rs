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

use crate::{
    cli::App,
    prelude::{Error, Result},
};
use derive_more::Display;
use std::{ops::Sub, str::FromStr};

pub trait ClippyCommand {
    #[allow(dead_code)]
    fn execute(&self, args: &App) -> Result<()>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
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

impl From<usize> for GreedyInt {
    fn from(data: usize) -> Self {
        Self { data }
    }
}

impl From<GreedyInt> for usize {
    fn from(data: GreedyInt) -> Self {
        data.data
    }
}

impl Sub<usize> for &GreedyInt {
    type Output = usize;

    fn sub(self, other: usize) -> usize {
        self.data - other
    }
}

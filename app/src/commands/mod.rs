pub mod completions;
pub mod list;
pub mod recall;
pub mod remove;
pub mod search;
pub mod store;
pub mod version;
pub mod wipe;

use std::{ops::Sub, str::FromStr};

pub use completions::GenCompletions;
use derive_more::Display;
pub use list::List;
pub use recall::Recall;
pub use remove::Remove;
pub use search::Search;
pub use store::Store;
pub use version::Version;
pub use wipe::Wipe;

use crate::{
    cli::ClippyCli,
    prelude::{Error, Result},
};

pub trait ClippyCommand {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        panic!(
            "Oops! you weren't supposed to be able to see this. \
            Please submit an issue here: \
            https://github.com/JustinBacher/clippy/issues"
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub struct GreedyInt(usize);

impl FromStr for GreedyInt {
    type Err = Error;

    fn from_str(s: &str) -> Result<GreedyInt> {
        Ok(GreedyInt(
            s.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse::<usize>()
                .unwrap(),
        ))
    }
}

impl From<GreedyInt> for usize {
    fn from(data: GreedyInt) -> Self {
        data.0
    }
}

impl Sub<usize> for &GreedyInt {
    type Output = usize;

    fn sub(self, other: usize) -> usize {
        self.0 - other
    }
}
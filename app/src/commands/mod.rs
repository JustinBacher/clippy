pub mod completions;
pub mod list;
pub mod recall;
pub mod remove;
pub mod search;
pub mod store;
pub mod version;
pub mod wipe;

use std::{ops::Sub, str::FromStr};

use anyhow::Result;
pub use completions::GenCompletions;
use derive_more::Display;
pub use list::List;
pub use recall::Recall;
pub use remove::Remove;
pub use search::Search;
pub use store::Store;
pub use version::Version;
pub use wipe::Wipe;

use crate::cli::ClippyCli;

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

use std::{error::Error, fmt};

#[derive(Debug)]
pub struct GreedyParseError;

impl Error for GreedyParseError {}

impl fmt::Display for GreedyParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

impl FromStr for GreedyInt {
    type Err = GreedyParseError;

    // Greedily accepts integers from the start of a string
    // until there are no numerical values
    fn from_str(s: &str) -> Result<GreedyInt, GreedyParseError> {
        let mut result = 0usize;
        let mut has_digits = false;

        for c in s.chars() {
            if let Some(digit) = c.to_digit(10) {
                has_digits = true;
                result = result
                    .checked_mul(10)
                    .and_then(|res| res.checked_add(digit as usize))
                    .ok_or(GreedyParseError)?;
            } else {
                break;
            }
        }

        if has_digits {
            Ok(GreedyInt(result))
        } else {
            Err(GreedyParseError)
        }
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

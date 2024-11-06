pub mod list;
pub mod recall;
pub mod remove;
pub mod search;
pub mod store;
pub mod version;
pub mod wipe;

pub(crate) use list::List;
pub(crate) use recall::Recall;
pub(crate) use remove::Remove;
pub(crate) use search::Search;
pub(crate) use store::Store;
pub(crate) use version::Version;
pub(crate) use wipe::Wipe;

use crate::cli::Cli;
use crate::prelude::Result;

pub trait ClippyCommand {
    #[allow(dead_code)]
    fn execute(&self, args: &Cli) -> Result<()>;
}

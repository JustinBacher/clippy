use super::ClippyCommand;

use crate::cli::Cli;
use crate::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Version;

impl ClippyCommand for Version {
    fn execute(&self, _: Cli) -> Result<()> {
        const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
        println!("{}", VERSION.unwrap_or("unknown"));

        Ok(())
    }
}

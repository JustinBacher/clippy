use clap::Parser;

use super::ClippyCommand;
use crate::{cli::ClippyCli, prelude::*};

#[derive(Parser, Debug, PartialEq)]
#[command()]
pub struct Version;

impl ClippyCommand for Version {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
        println!("{}", VERSION.unwrap_or("unknown"));

        Ok(())
    }
}

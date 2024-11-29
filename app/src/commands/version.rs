use anyhow::Result;
use clap::Parser;

use super::ClippyCommand;
use crate::cli::ClippyCli;

#[derive(Parser, Debug, PartialEq)]
#[command()]
pub struct Version;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

impl ClippyCommand for Version {
    fn execute(&self, _: &ClippyCli) -> Result<()> {
        println!("{}", VERSION.unwrap_or("unknown"));

        Ok(())
    }
}

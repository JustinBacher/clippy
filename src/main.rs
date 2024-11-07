extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod cli;
mod commands;
mod error;
mod prelude;
mod utils;

use crate::commands::ClippyCommand;
use clap::Parser;
use cli::{Cli, Commands};
use prelude::Result;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = Cli::parse();

    match &args.command {
        Commands::Store(command) => command.execute(&args)?,
        Commands::List(command) => command.execute(&args)?,
        Commands::Recall(command) => command.execute(&args)?,
        Commands::Search(command) => command.execute(&args)?,
        Commands::Wipe(command) => command.execute(&args)?,
        Commands::Remove(command) => command.execute(&args)?,
        Commands::Version(command) => command.execute(&args)?,
    }

    Ok(())
}

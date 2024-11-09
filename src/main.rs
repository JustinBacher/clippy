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
use cli::{App, Commands};
use prelude::Result;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let args = App::parse();

    // I wanna know if there's a better way to do this than this huge blob
    //
    // But at least this way the commands are in there own files and it's
    // easier to destinguish tests
    match &args.command {
        Commands::GenCompletions(command) => command.execute(&args)?,
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

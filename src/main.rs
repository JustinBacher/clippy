pub use clippy::cli;
pub use clippy::commands;
pub use clippy::error;
pub use clippy::prelude;
pub use clippy::utils;

use clap::Parser;
use clippy::cli::{App, Commands};
use clippy::commands::ClippyCommand;
use clippy::prelude::Result;

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

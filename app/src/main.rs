use anyhow::Result;
use clap::Parser;
pub use clippy::{cli, commands};
use clippy::{
    cli::{ClippyCli, Commands},
    commands::ClippyCommand,
};

fn main() -> Result<()> {
    let args = ClippyCli::parse();

    // I wanna know if there's a better way to do this than this huge blob
    //
    // But at least this way the commands are in there own files and it's
    //
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

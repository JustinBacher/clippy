extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod cli;
mod commands;
mod error;
mod prelude;
mod utils;

use crate::commands::ClippyCommand;
use camino::Utf8PathBuf;
use clap::Parser;
use cli::{Cli, Commands};
use prelude::Result;
use redb::{Database, ReadableTable};
use std::{
    cell::RefCell,
    io::{stdout, Stdout, Write},
};
use utils::{database::TABLE_DEF, formatting::format_entry};

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
    }

    Ok(())
}

fn wipe(db_path: &Utf8PathBuf) -> Result<()> {
    let db = Database::open(db_path)?;
    let tx = db.begin_write()?;
    {
        let table = RefCell::new(tx.open_table(TABLE_DEF)?);

        table.borrow().iter()?.for_each(|entry| {
            table.borrow_mut().remove(entry.unwrap().0.value()).unwrap();
        });
    }
    tx.commit()?;
    Ok(())
}

mod cli;
mod error;
mod prelude;
mod utils;

use camino::Utf8PathBuf;
use chrono::Local;
use clap::Parser;
use cli::{Cli, Commands};
use prelude::*;
use redb::{Database, ReadableTable};
use std::{
    env,
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
};
use utils::database::{remove_duplicates, TABLE};
use utils::formatting::*;

const MAX_PAYLOAD_SIZE: usize = 5e6 as usize;

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Store {} => match env::var("CLIPBOARD_STATE").unwrap().as_str() {
            "sensitive" | "clear" => (),
            _ => {
                let db_path = args.db_path;
                store(&db_path, &mut stdin())?;
                remove_duplicates(&db_path, args.duplicates)?;
            }
        },
        Commands::List { include_dates } => list(
            &args.db_path,
            &mut stdout(),
            args.preview_width,
            include_dates.unwrap(),
        )?,
    }

    Ok(())
}

fn store(db_path: &Utf8PathBuf, input: &mut Stdin) -> Result<()> {
    let mut payload = Vec::new();
    input.read_to_end(&mut payload)?;

    if std::mem::size_of_val(&payload) > MAX_PAYLOAD_SIZE || trim(&payload).len() == 0 {
        return Ok(());
    }

    let db = Database::create(&db_path)?;
    let tx = db.begin_write()?;
    {
        let mut table = tx.open_table(TABLE)?;
        table.insert(Local::now().timestamp_millis(), payload.to_owned())?;
    }
    tx.commit()?;

    Ok(())
}

fn list(db_path: &Utf8PathBuf, out: &mut Stdout, width: usize, include_dates: bool) -> Result<()> {
    let db = Database::create(&db_path)?;
    let tx = db.begin_read()?;
    {
        let table = tx.open_table(TABLE)?;
        let count = table.iter()?.count();

        table.iter()?.enumerate().for_each(|(i, entry)| {
            let (date, payload) = format_entry(entry.unwrap(), width);

            match include_dates {
                true => out.write_fmt(format_args!("{}. {}:\t{}\n", count - i, date, payload)),
                false => out.write_fmt(format_args!("{}. {}\n", count - i, payload)),
            }
            .unwrap();
        });
    }

    Ok(())
}

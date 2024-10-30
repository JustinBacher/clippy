mod cli;
mod error;
mod prelude;
mod utils;

use camino::Utf8PathBuf;
use chrono::{DateTime, Local};
use clap::Parser;
use cli::{Cli, Commands};
use prelude::Result;
use redb::{Database, ReadableTable, TableDefinition};
use std::cell::RefCell;
use std::env;
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use utils::trim_ascii_whitespace;

const TABLE: TableDefinition<i64, Vec<u8>> = TableDefinition::new("clips");
const MAX_PAYLOAD_SIZE: usize = 5e6 as usize;

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Store {} => match env::var("CLIPBOARD_STATE").unwrap().as_str() {
            "sensitive" | "clear" => (),
            _ => store(
                &args.db_path,
                &mut stdin(),
                args.max_dedupe_search,
                args.max_items,
            )?,
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

fn store(
    db_path: &Utf8PathBuf,
    input: &mut Stdin,
    max_dedupe_search: usize,
    max_items: usize,
) -> Result<()> {
    let mut payload = Vec::new();
    input.read_to_end(&mut payload)?;

    if std::mem::size_of_val(&payload) > MAX_PAYLOAD_SIZE
        || trim_ascii_whitespace(&payload).len() == 0
    {
        return Ok(());
    }

    let db = Database::create(&db_path)?;
    let tx = db.begin_write()?;
    {
        let table = RefCell::new(tx.open_table(TABLE)?);

        table
            .borrow_mut()
            .insert(Local::now().timestamp_millis(), payload.to_owned())?;

        table
            .borrow()
            .iter()?
            .rev()
            .take(max_dedupe_search)
            .filter(|entry| entry.as_ref().unwrap().1.value() == payload)
            .for_each(|entry| {
                table.borrow_mut().remove(entry.unwrap().0.value()).unwrap();
            });

        table
            .borrow()
            .iter()?
            .by_ref()
            .skip(max_items - max_dedupe_search)
            .for_each(|entry| {
                table.borrow_mut().remove(entry.unwrap().0.value()).unwrap();
            });
    }
    tx.commit()?;

    Ok(())
}

fn list(
    db_path: &Utf8PathBuf,
    out: &mut Stdout,
    preview_width: usize,
    include_dates: bool,
) -> Result<()> {
    let db = Database::create(&db_path)?;

    let tx = db.begin_read()?;

    {
        let table = tx.open_table(TABLE)?;
        let count = table.iter()?.count();

        table
            .iter()?
            .take(preview_width)
            .enumerate()
            .for_each(|(i, entry)| {
                let (date, payload) = entry.unwrap();
                let copied = String::from_utf8(payload.value()).unwrap();
                let pretty_date = DateTime::from_timestamp_millis(date.value())
                    .unwrap()
                    .format("%c");

                match include_dates {
                    true => out.write_fmt(format_args!(
                        "{}. {}:\t{}\n",
                        count - i,
                        pretty_date,
                        copied,
                    )),
                    false => out.write_fmt(format_args!("{}. {}\n", count - i, copied)),
                }
                .unwrap();
            });
    }

    Ok(())
}

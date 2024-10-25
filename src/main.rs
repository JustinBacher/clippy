use camino::Utf8PathBuf;
use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use dirs;
use jammdb::{Error, DB};
use log::info;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Read, Write};
use std::mem::size_of_val;

const TABLE_NAME: &str = "history";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct CopiedData {
    date: DateTime<Local>,
    data: Vec<u8>,
}

#[derive(Parser)]
#[command(name = "clippy")]
struct Cli {
    #[arg(short, long, default_value = dirs::cache_dir().unwrap().join("clippy").join("config").into_os_string())]
    config_path: Utf8PathBuf,
    #[arg(short, long, default_value = dirs::config_dir().unwrap().join("clippy").join("db").into_os_string())]
    db_path: Utf8PathBuf,
    #[arg(default_value = "100")]
    max_dedupe_search: u32,
    #[arg(default_value = "750")]
    max_items: u32,
    #[arg(default_value = "100")]
    preview_width: u16,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Store {},
    List { include_dates: String },
}

fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match args.command {
        Commands::Store {} => match env::var("CLIPBOARD_STATE").unwrap().as_str() {
            "sensitive" | "clear" => return Ok(()),
            _ => store(
                &args.db_path,
                &mut io::stdin(),
                args.max_dedupe_search,
                args.max_items,
            )?,
        },
        Commands::List { .. } => list(&args.db_path, &mut io::stdout(), args.preview_width)?,
    }

    Ok(())
}

fn trim_ascii_whitespace(x: &[u8]) -> &[u8] {
    let from = match x.iter().position(|x| !x.is_ascii_whitespace()) {
        Some(i) => i,
        None => return &x[0..0],
    };
    let to = x.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
    &x[from..=to]
}

fn store(
    db_path: &Utf8PathBuf,
    input: &mut io::Stdin,
    max_dedupe_search: u32,
    max_items: u32,
) -> Result<(), Error> {
    let mut copied: Vec<u8> = Vec::new();
    if !input.read_to_end(&mut copied).is_ok() {
        return Ok(());
    }

    // Do not store larger than 5Mb or empty values
    if size_of_val(&*copied) > (5.0 * 1e6) as usize {
        return Ok(());
    }

    if trim_ascii_whitespace(&copied).len() == 0 {
        return Ok(());
    }

    let db = DB::open(&db_path)?;
    {
        let tx = db.tx(true)?;
        let table = tx.get_or_create_bucket(TABLE_NAME)?;
        table.put(Local::now().timestamp().to_le_bytes(), copied.clone())?;
        tx.commit()?;
    }
    {
        let tx = db.tx(true)?;
        let table = tx.get_bucket(TABLE_NAME)?;
        let mut seen = 0;

        table
            .kv_pairs()
            .into_iter()
            .enumerate()
            .filter(|(i, kv)| {
                if kv.value() == copied {
                    seen += 1
                }
                *i > max_items as usize || seen >= (max_dedupe_search + 1) as usize
            })
            .for_each(|(_, kv)| {
                table.delete(kv.key()).unwrap();
            });
        tx.commit()?;
    }
    Ok(())
}

fn list(db_path: &Utf8PathBuf, out: &mut io::Stdout, preview_width: u16) -> Result<(), Error> {
    let db = DB::open(&db_path)?;
    let tx = db.tx(false)?;

    let copied = tx.get_or_create_bucket(TABLE_NAME)?;
    copied
        .kv_pairs()
        .into_iter()
        .take(preview_width as usize)
        .for_each(|kv| {
            let date = Local.timestamp_opt(i64::from_le_bytes(kv.key().try_into().unwrap()), 0);
            out.write_fmt(format_args!("{date:?}")).unwrap();
        });
    Ok(())
}

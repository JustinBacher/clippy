mod cli;
mod clip;
mod error;
mod prelude;

use camino::Utf8PathBuf;
use chrono::Local;
use chrono_humanize::{Accuracy, HumanTime, Tense};
use clap::Parser;
use cli::{Cli, Commands};
use clip::Clip;
use prelude::Result;
use redb::{Database, ReadableTable, TableDefinition};
use std::env;
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use std::mem::size_of_val;
use std::cmp::min;

const TABLE: TableDefinition<&str, u64> = TableDefinition::new("clipboard");

#[derive(Parser)]
#[command(name = "clippy")]
struct Cli {
    #[arg(short, long, default_value = dirs::config_dir().unwrap().join("clippy").join("config").into_os_string())]
    config_path: Utf8PathBuf,
    
    #[arg(short, long, default_value = dirs::cache_dir().unwrap().join("clippy").join("db").into_os_string())]
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
const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("clips");
const MAX_PAYLOAD_SIZE: usize = 5e6 as usize;

fn main() -> Result<()> {
    let args = Cli::parse();
    println!("Dir: {}", args.db_path);

    match args.command {
        Commands::Store {} => match env::var("CLIPBOARD_STATE").unwrap().as_str() {
            "sensitive" | "clear" => return Ok(()),
            _ => store(
                &args.db_path,
                &mut stdin(),
                args.max_dedupe_search,
                args.max_items,
            )?,
        },
        Commands::List { .. } => list(&args.db_path, &mut stdout(), args.preview_width)?,
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
    input: &mut Stdin,
    max_dedupe_search: usize,
    max_items: usize,
) -> Result<()> {
    let mut payload = Vec::new();
    input.read_to_end(&mut payload)?;

    if (size_of_val(&payload) > MAX_PAYLOAD_SIZE) || (trim_ascii_whitespace(&payload).len() == 0) {
        return Ok(());
    }

    let db = Database::create(&db_path)?;

    let tx = db.begin_write()?;
    {
        let table = tx.open_table(TABLE)?;

        let clip = Clip {
            date: Local::now(),
            payload: payload.clone(),
        };
        table.insert(clip.date.timestamp().to_le_bytes(), clip.into())?;

        let mut cursor = table.kv_pairs();

        cursor
            .by_ref()
            .take(max_dedupe_search)
            .filter(|clip| clip.value() == payload)
            .for_each(|clip| {
                table.delete(clip.key()).unwrap();
            });

        cursor.skip(max_items - max_dedupe_search).for_each(|clip| {
            table.delete(clip.key()).unwrap();
        })
    }
    tx.commit()?;

    Ok(())
}

fn list(db_path: &Utf8PathBuf, out: &mut Stdout, preview_width: usize) -> Result<()> {
    let db = DB::open(&db_path)?;
    let tx = db.tx(false)?;

    {
        let table = tx.get_bucket(TABLE_NAME)?;
        let count = table.kv_pairs().count();

        table
            .kv_pairs()
            .take(preview_width)
            .enumerate()
            .for_each(|(i, kv)| {
                let clip = Clip::from(kv.value());
                out.write_fmt(format_args!(
                    "{}. {}:\t{}\n",
                    count - i,
                    HumanTime::from(clip.date)
                        .to_text_en(Accuracy::Rough, Tense::Past)
                        .to_string(),
                    std::str::from_utf8(&clip.payload).unwrap().to_string()
                ))
                .unwrap();
            });
    }
    Ok(())
}

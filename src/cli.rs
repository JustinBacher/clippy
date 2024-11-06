use crate::commands;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum, ValueHint::AnyPath};
use dirs::cache_dir;
use serde::Serialize;
use std::path::Path;

#[derive(ValueEnum, Parser, Clone, Default, PartialEq, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardState {
    #[default]
    Nil,
    Data,
    Clear,
    Sensitive,
}

#[derive(Parser)]
#[command(name = "clippy", version)]
/// Clippy is a lightweight clipboard history manager for Wayland built using Rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long,
        default_value = Path::join(&cache_dir().unwrap(), "/clippy/db") .to_str() .unwrap() .to_string(),
        value_hint(AnyPath))]

    /// Path to the local database used to store previous clips
    pub db_path: Utf8PathBuf,

    #[arg(short, long, alias("dupes"), default_value = "0")]
    /// Number of most recent duplicates to keep. Negative values remove x oldest duplicates instead.
    ///
    /// Positive values keep x amount of most recent duplicates.
    /// Negative values remove x amount of duplicates from the end.
    /// 0 will retain only unique clips. Removing any duplicates.
    pub duplicates: i32,

    #[arg(short, long, default_value = "1000")]
    /// Amount of clips to keep in database
    pub keep: usize,

    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Parser, Debug, PartialEq)]
/// Lists all stored clips in clipboard
pub struct List {
    #[arg(short('d'), long, action)]
    /// Includes dates clips were taken in the output
    include_dates: bool,

    #[arg(short('w'), long, default_value = "100")]
    /// Max characters to show of clips in preview. Use 0 to retain original width.
    ///
    /// This does not affect what is put back into the clipboard
    preview_width: usize,
}

#[derive(Parser, Debug, PartialEq)]
#[command(allow_missing_positional(true))]
/// Outputs clip to `stdout`.
///
/// Meant for use with `wl-paste`
pub struct Recall {
    #[arg()]
    /// The id of the clip to use.
    ///
    /// From the output of `list` command
    id: GreedyInt,
    #[arg()]
    other: Option<Vec<String>>,
}

#[derive(Parser, Debug, PartialEq)]
/// Search for a clip that contains `query`
pub struct Search {
    #[arg(short, long)]
    /// The query to search for in clipboard history
    query: String,

    #[arg(short('d'), long, action)]
    /// Includes dates clips were taken in the output
    include_dates: bool,

    #[arg(short('w'), long, default_value = "100")]
    /// Max characters to show of clips in preview. Use 0 to retain original width.
    ///
    /// This does not affect what is put back into the clipboard
    preview_width: usize,
}

#[derive(Parser, Debug, PartialEq)]
/// Wipes clipboard history from database
pub struct Wipe {}

#[derive(Parser, Debug, PartialEq)]
/// Removes a clip from the database
pub struct Remove {
    /// The id of the clip from the output of list command
    id: Option<usize>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    Store(commands::Store),
    List(commands::List),
    Recall(commands::Recall),
    Search(commands::Search),
    Wipe(commands::Wipe),
    Remove(commands::Remove),
}

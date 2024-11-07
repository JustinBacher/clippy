use crate::{commands, utils::get_config_path};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueHint::AnyPath};

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    Store(commands::Store),
    GenCompletions(commands::GenCompletions),
    List(commands::List),
    Recall(commands::Recall),
    Search(commands::Search),
    Wipe(commands::Wipe),
    Remove(commands::Remove),
    Version(commands::Version),
}

#[derive(Parser)]
#[command(name = "clippy", version)]
/// Clippy is a lightweight clipboard history manager for Wayland built using Rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg( long, default_value = get_config_path().unwrap(), value_hint(AnyPath))]

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

use std::path::Path;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueHint::AnyPath};

use crate::{commands, utils::get_cache_path};

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    GenCompletions(commands::GenCompletions),
    List(commands::List),
    Pair(commands::Pair),
    Recall(commands::Recall),
    Remove(commands::Remove),
    Search(commands::Search),
    Store(commands::Copy),
    Version(commands::Version),
    Wipe(commands::Wipe),
}

pub const APP_NAME: &str = "clippy";

#[derive(Parser)]
#[command(name = APP_NAME, version)]
/// Clippy is a lightweight clipboard history manager for Wayland built using
/// Rust
pub struct ClippyCli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long,
        default_value = get_cache_path(Path::new("")).unwrap().as_path().to_str().unwrap().to_string(),
        value_hint(AnyPath)
    )]
    /// Path to the local database used to store previous clips
    pub db_path: Utf8PathBuf,

    /// Number of most recent duplicates to keep. Negative values remove x
    /// oldest duplicates instead.
    ///
    /// Positive values keep x amount of most recent duplicates.
    /// Negative values remove x amount of duplicates from the end.
    /// 0 will retain only unique clips. Removing any duplicates.
    #[arg(short, long, alias("dupes"), default_value = "0")]
    pub duplicates: i64,

    /// Amount of clips to keep in database
    #[arg(short, long, default_value = "1000")]
    pub keep: u64,

    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[cfg(test)]
pub fn mock_cli<'a, I>(args: I) -> Option<ClippyCli>
where
    I: Iterator<Item = &'a str>,
{
    ClippyCli::try_parse_from(std::iter::once(APP_NAME).chain(args)).ok()
}

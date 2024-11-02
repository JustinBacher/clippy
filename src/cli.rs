use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use dirs;

#[derive(Parser)]
#[command(name = "clippy")]
/// Lightweight clipboard history manager for Wayland built using Rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long("db"), default_value = dirs::cache_dir().unwrap().join("clippy").join("db").into_os_string())]
    /// Path to the local database used to store previous clips
    pub db_path: Utf8PathBuf,

    #[arg(short, default_value = "10")]
    /// Number of duplicates to keep. Negative values remove x oldest duplicates instead.
    ///
    /// Positive values keep x amount of most recent duplicates
    /// Negative values remove x amount of duplicates from the end
    /// 0 removes all duplicates
    pub duplicates: i32,

    #[arg(default_value = "1000")]
    /// Amount of clips to keep in database
    pub keep: usize,
}

#[derive(Subcommand)]
pub enum Commands {
    Store {},
    List {
        #[arg(short('d'), long)]
        /// Disables outputting the dates clips were taken in the output
        exclude_dates: bool,
        #[arg(short('w'), long, default_value = "100")]
        preview_width: usize,
    },
}

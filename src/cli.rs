use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use dirs;

#[derive(Parser)]
#[command(name = "clippy")]
/// Lightweight clipboard history manager for Wayland built using Rust
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long("config"), default_value = dirs::config_dir().unwrap().join("clippy").join("config").into_os_string())]
    /// Path to the config file used to set defaults for commands
    pub config_path: Utf8PathBuf,

    #[arg(short, long("db"), default_value = dirs::cache_dir().unwrap().join("clippy").join("db").into_os_string())]
    /// Path to the local database used to store previous clips
    pub db_path: Utf8PathBuf,

    #[arg(short, default_value = "10")]
    /// Number of duplicates to keep. Negative values remove x oldest dupes instead.
    ///
    /// Positive values keep x amount of most recent duplicates
    /// Negative values remove x amount of duplicates from the end
    /// 0 keeps all duplicates
    pub duplicates: i32,

    #[arg(default_value = "1000")]
    /// Amount of clips to keep in database
    pub keep: usize,

    #[arg(default_value = "100")]
    /// How many characters to use before truncating when displaying clips (ex: using list)
    pub preview_width: usize,
}

#[derive(Subcommand)]
pub enum Commands {
    Store {},
    List {
        #[arg(short, long, default_value = "false")]
        /// Displays the date clips were taken in the output.
        include_dates: Option<bool>,
    },
}

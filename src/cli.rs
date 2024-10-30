use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use dirs;

#[derive(Parser)]
#[command(name = "clippy")]
pub struct Cli {
    #[arg(short, long, default_value = dirs::config_dir().unwrap().join("clippy").join("config").into_os_string())]
    pub config_path: Utf8PathBuf,

    #[arg(short, long, default_value = dirs::cache_dir().unwrap().join("clippy").join("db").into_os_string())]
    pub db_path: Utf8PathBuf,

    #[arg(default_value = "100")]
    pub max_dedupe_search: usize,

    #[arg(default_value = "750")]
    pub max_items: usize,

    #[arg(default_value = "100")]
    pub preview_width: usize,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Store {},
    List {
        #[arg(default_value = "false")]
        include_dates: Option<bool>,
    },
}

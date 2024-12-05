use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use clippy_daemon::utils::{
    clipboard::listen_to_clipboard,
    config::{Config, watch_config},
    get_config_path,
};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("clippy", "config.toml").unwrap();
    let config = Arc::new(Mutex::new(
        Config::from_file(Path::new(&config_path)).await?,
    ));
    let watcher_task = task::spawn(watch_config(config_path, Arc::clone(&config)));

    listen_to_clipboard(config).await?;

    let _ = watcher_task.await;

    Ok(())
}

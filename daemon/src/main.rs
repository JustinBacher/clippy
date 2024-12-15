use std::{path::Path, sync::Arc};

use anyhow::Result;
use tokio::{sync::Mutex, task};

use clippy_daemon::{
    sync::connection::DistributedHashNetwork,
    utils::{
        clipboard::listen_to_clipboard,
        config::{watch_config, Config},
        get_config_path,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("config.toml").unwrap();
    let config = Arc::new(Mutex::new(
        Config::from_file(Path::new(&config_path)).await?,
    ));
    let watcher_task = task::spawn(watch_config(
        config_path.to_string_lossy().to_string(),
        Arc::clone(&config),
    ));

    let dht = DistributedHashNetwork::new(config.clone()).await?;
    dht.start_server().await?;

    listen_to_clipboard(config).await?;

    let _ = watcher_task.await;

    Ok(())
}

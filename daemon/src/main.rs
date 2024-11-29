use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use camino::Utf8Path;
use clippy_daemon::{
    database::{ensure_db_size, get_db, remove_duplicates},
    platforms::listen_for_clips,
    utils::{
        async_helpers::GeneratorStream,
        config::{watch_config, Config},
        get_cache_path, get_config_path,
    },
};
use futures::StreamExt;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("clippy", "config.toml").unwrap();
    let config = Arc::new(Mutex::new(
        Config::from_file(Path::new(&config_path)).await?,
    ));
    let watcher_task = {
        let config_path = config_path.clone();
        let config = Arc::clone(&config);
        task::spawn(watch_config(config_path, config))
    };

    respond_to_clips().await?;

    let _ = watcher_task.await;

    Ok(())
}

async fn respond_to_clips() -> Result<()> {
    let path = get_cache_path("clippy", "db").unwrap();
    let db = get_db(Utf8Path::new(path.as_str()))?;
    let generator = listen_for_clips().await?;
    let mut stream = GeneratorStream::new(generator);

    while let Some(clip) = stream.next().await {
        let tx = db.rw_transaction()?;
        {
            tx.insert(clip)?;

            // TODO: get these numbers from config
            remove_duplicates(&db, 10)?;
            ensure_db_size(&db, 100)?;
        }
        tx.commit()?;
    }
    Ok(())
}

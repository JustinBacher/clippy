mod database;
mod utils;

use std::{path::Path, sync::{mpsc, Arc, Mutex}};

use anyhow::Result;
use log::info;
use notify::{event::ModifyKind, Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio_stream::Stream;

use clippy_daemon::platforms;
use crate::{
    database::{get_db, remove_duplicates, ClipEntry},
    platforms::listen_for_clips
};
use utils::{config::{Config, watch_config}, async_helpers::GeneratorStream, get_cache_path, get_config_path};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("clippy", "config.toml").unwrap();
    let config = Arc::new(Mutex::new(
        Config::from_file(Path::new(&config_path)).await,
    ));
    let watcher_task = {
        let config_path = config_path.clone();
        let config = Arc::clone(&config);
        tokio::spawn(config.watch(config_path))
    };

    respond_to_clips().await;

    let _ = watcher_task.await;

    Ok(())
}

async fn respond_to_clips() -> Result<()> {
    let db = get_db(&get_cache_path("clippy", "db").unwrap())?;
    let generator = listen_for_clips().await?;
    let mut stream = GeneratorStream::new(generator);

    while let Some(clip) = stream.next().await {
        let tx = db.rw_transaction()?;
        {
            tx.insert(clip)?;

            // TODO: get these numbers from config
            remove_duplicates(&db, 10)?;
            ensure_db_size(&db, 100)
        }
        tx.commit()?;
    }
    Ok(())
}

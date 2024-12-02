use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use camino::Utf8Path;
use clippy_daemon::{
    database::{ensure_db_size, get_db, remove_duplicates},
    utils::{
        async_helpers::GeneratorStream,
        clipboard::listen_to_clipboard,
        config::{Config, watch_config},
        get_cache_path,
        get_config_path,
    },
};
use tokio::task;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("clippy", "config.toml").unwrap();
    let config = Arc::new(Mutex::new(Config::from_file(Path::new(&config_path)).await?));
    let watcher_task = task::spawn(watch_config(config_path, Arc::clone(&config)));

    respond_to_clips(config).await?;

    let _ = watcher_task.await;

    Ok(())
}

async fn respond_to_clips(config: Arc<Mutex<Config>>) -> Result<()> {
    let db_path = get_cache_path("clippy", "db").unwrap();
    let db = get_db(Utf8Path::new(&db_path))?;
    let clips = listen_to_clipboard(Arc::clone(&config)).await;
    let mut generator = GeneratorStream::new(Box::new(clips));

    while let Some(Ok(clip)) = generator.next().await {
        let config_guard = config.lock().unwrap();
        let config_clipboard = &config_guard.clipboard;
        for board in config_clipboard.values() {
            let tx = db.rw_transaction()?;
            {
                if board.clone().can_store(&clip).unwrap() {
                    tx.insert(clip.clone())?;
                    remove_duplicates(&db, &board.remove_duplicates, &board.keep_duplicates)?;
                    ensure_db_size(&db, &board.max_size.unwrap_or(1000))?;
                }
            }
            tx.commit()?;
        }
    }
    Ok(())
}

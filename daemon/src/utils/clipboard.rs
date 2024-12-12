use std::sync::Arc;

use anyhow::{anyhow, Result};
use arboard::Clipboard;
use camino::Utf8Path;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{
    database::clipboard::{ensure_db_size, get_db, remove_duplicates, ClipEntry},
    utils::{config::Config, get_cache_path},
};

pub async fn listen_to_clipboard(config: Arc<Mutex<Config>>) -> Result<()> {
    let mut clipboard = Clipboard::new().unwrap();
    let mut previous_content = clipboard
        .get()
        .text()
        .map_err(|e| anyhow!("Unable to obtain clipboard. {e}"))
        .unwrap();

    loop {
        let new_content = clipboard
            .get()
            .text()
            .map_err(|e| anyhow!("Unable to obtain clipboard. {e}"))
            .unwrap();

        if previous_content != new_content {
            respond_to_clip(
                &Arc::clone(&config),
                ClipEntry::new(previous_content.as_bytes()),
            )
            .await?;
            previous_content = new_content;
        }

        let millis = config.lock().await.general.as_ref().unwrap().polling_rate.unwrap_or_default();
        sleep(Duration::from_millis(millis)).await;
    }
}

pub async fn respond_to_clip(config: &Arc<Mutex<Config>>, clip: ClipEntry) -> Result<()> {
    let config_guard = config.lock().await;
    let config_clipboard = &config_guard.clipboard;
    for (mut name, board) in config_clipboard.iter() {
        let pri = "primary".to_string();
        if *name == "default" {
            name = &pri;
        }

        let db_path = get_cache_path(name)?;
        let db = get_db(Utf8Path::new(db_path.as_path().to_str().unwrap()))?;
        let tx = db.rw_transaction()?;
        {
            if board.clone().can_store(&clip).unwrap() {
                println!("Stored: {:?}", clip);
                tx.insert(clip.clone())?;
                remove_duplicates(&db, &board.remove_duplicates, &board.keep_duplicates)?;
                ensure_db_size(&db, &board.max_size.unwrap_or(1000))?;
            }
        }
        tx.commit()?;
    }
    Ok(())
}

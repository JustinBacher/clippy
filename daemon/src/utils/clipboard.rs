use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use arboard::Clipboard;
use genawaiter::{Generator, sync::gen, yield_};
use tokio::time::{Duration, sleep};

use crate::{database::ClipEntry, utils::config::Config};

pub async fn listen_to_clipboard(config: Arc<Mutex<Config>>) -> impl Generator<Yield = Result<ClipEntry>, Return = ()> {
    let mut clipboard = Clipboard::new().unwrap();
    let mut previous_content = clipboard
        .get()
        .text()
        .map_err(|e| anyhow!("Unable to obtain clipboard. {e}"))
        .unwrap();

    gen!({
        let new_content = clipboard
            .get()
            .text()
            .map_err(|e| anyhow!("Unable to obtain clipboard. {e}"))
            .unwrap();

        if previous_content != new_content {
            yield_!(Ok(ClipEntry::new(previous_content.as_bytes())));
            previous_content = new_content;
        }

        let millis = config
            .lock()
            .unwrap()
            .general
            .as_ref()
            .unwrap()
            .polling_rate
            .unwrap_or_default();
        sleep(Duration::from_millis(millis)).await;
    })
}

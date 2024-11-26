mod database;
mod utils;

use futures::StreamExt;
use genawaiter::GeneratorState;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

use anyhow::Result;
use genawaiter::Generator;
use log::info;
use notify::{event::ModifyKind, Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio_stream::Stream;

pub use clippy_daemon::platforms;

use crate::database::{get_db, remove_duplicates, ClipEntry};
use crate::platforms::listen_for_clips;
use utils::config::ConfigStruct;
use utils::{get_cache_path, get_config_path};

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = get_config_path("clippy", "config.toml").unwrap();
    let config = Arc::new(Mutex::new(
        ConfigStruct::from_file(Path::new(&config_path)).await,
    ));
    let watcher_task = {
        let config_path = config_path.clone();
        let config = Arc::clone(&config);
        tokio::spawn(watch_config(config_path, config))
    };

    respond_to_clips().await;

    let _ = watcher_task.await;

    Ok(())
}

async fn watch_config(config_path: String, config: Arc<Mutex<ConfigStruct>>) {
    // Create a channel to receive events from the watcher.
    let (tx, rx) = mpsc::channel();

    // Create a watcher using `notify`.
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )
    .expect("Failed to create watcher");

    // Start watching the config file.
    watcher
        .watch(Path::new(&config_path), RecursiveMode::NonRecursive)
        .expect("Failed to watch file");

    println!("Watching config file: {}", config_path);

    // Handle events asynchronously.
    while let Ok(event) = rx.recv() {
        if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
            info!("Config file changed, reloading...");

            let new_config = ConfigStruct::from_file(Path::new(&config_path)).await;
            let mut config_guard = config.lock().unwrap();
            *config_guard = new_config;

            info!("Config updated: {:?}", *config_guard);
        }
    }
}

async fn respond_to_clips() -> Result<()> {
    let db = get_db(&get_cache_path("clippy", "db").unwrap())?;
    let generator = listen_for_clips().await?;
    let mut stream = GeneratorStream::new(generator);

    while let Some(clip) = stream.next().await {
        let tx = db.rw_transaction()?;
        {
            tx.insert(clip)?;
            remove_duplicates(&db, 10)?;
            tx.commit()?;
        }
    }
    Ok(())
}

struct GeneratorStream {
    generator: Pin<Box<dyn Generator<Yield = ClipEntry, Return = ()>>>,
}

impl GeneratorStream {
    fn new(generator: Box<dyn Generator<Yield = ClipEntry, Return = ()>>) -> Self {
        Self {
            generator: generator.into(),
        }
    }
}

impl Stream for GeneratorStream {
    type Item = ClipEntry;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Use the generator's resume method to drive it
        let this = self.get_mut();
        match this.generator.as_mut().resume() {
            GeneratorState::Yielded(clip) => Poll::Ready(Some(clip)),
            GeneratorState::Complete(_) => Poll::Ready(None),
        }
    }
}

use std::{collections::HashMap, path::Path};
use serde::Deserialize;
use tokio::fs;
use camino::Utf8Path;

#[derive(Debug, Deserialize)]
struct Preview {
    width: Option<i64>,
    include_dates: Option<bool>
}

#[derive(Debug, Deserialize)]
struct Exclude {
    application: Option<String>,
    pattern: Option<String>,
    mime_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct General {
    db_path: Option<Utf8Path>,
    max_size: Option<u64>,
    keep_duplicates: Option<u64>,
    remove_duplicates: Option<u64>,
    preview: Option<HashMap<String, Preview>>,
    exclude: Option<HashMap<String, Exclude>>,
}

#[derive(Debug, Deserialize)]
struct Config {
    polling_rate: Option<usize>,
    timeout_rate: Option<usize>,
    clipboard: Option<HashMap<String, General>>,
}

impl Config {
    pub async fn from_file(path: &Path) -> Self {
        let content = fs::read_to_string(path).await.expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse config file")
    }

    async fn watch(path: String, config: Arc<Mutex<Config>>) {
        let (tx, rx) = mpsc::channel();
    
        RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        )
        .expect("Failed to create watcher")
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .expect("Failed to watch file");
    
        while let Ok(event) = rx.recv() {
            if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
                info!("Config file changed, reloading...");
    
                let new_config = ConfigStruct::from_file(Path::new(&path)).await;
                let mut config_guard = config.lock().unwrap();
                *config_guard = new_config;
    
                info!("Config updated: {:?}", *config_guard);
            }
        }
    }
}
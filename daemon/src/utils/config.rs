use std::{
    collections::HashMap,
    path::Path,
    sync::{mpsc, Arc, Mutex},
};

use anyhow::{anyhow, Result};
use log::debug;
use notify::{
    event::ModifyKind, Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::get_cache_path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Preview {
    pub width: Option<u64>,
    pub include_dates: Option<bool>,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            width: Some(100),
            include_dates: Some(true),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Clude {
    pub applications: Option<String>,
    pub patterns: Option<String>,
    pub mime_types: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct General {
    pub db_path: Option<String>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            db_path: get_cache_path("clippy", "db"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Clipboard {
    pub max_size: Option<u64>,
    pub keep_duplicates: Option<u64>,
    pub remove_duplicates: Option<u64>,
    pub preview: Option<HashMap<String, Preview>>,
    pub exclude: Option<HashMap<String, Clude>>,
    pub include: Option<HashMap<String, Clude>>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            max_size: Some(1_000),
            keep_duplicates: Some(10),
            remove_duplicates: Some(0),
            preview: Some(HashMap::from([("default".to_string(), Preview::default())])),
            exclude: Some(HashMap::from([("default".to_string(), Clude::default())])),
            include: Some(HashMap::from([("default".to_string(), Clude::default())])),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: Option<General>,
    pub polling_rate: Option<usize>,
    pub timeout_rate: Option<usize>,
    pub clipboard: Option<HashMap<String, Clipboard>>,
}

impl Config {
    pub async fn from_file(path: &Path) -> Result<Self> {
        fs::read_to_string(path)
            .await
            .map_err(|err| anyhow!("Failed to read config file: {err}"))
            .and_then(|content| {
                toml::from_str(&content).map_err(|err| anyhow!("Failed to parse TOML: {err}"))
            })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: Some(General::default()),
            polling_rate: Some(100),
            timeout_rate: Some(300),
            clipboard: Some(HashMap::from([(
                "default".to_string(),
                Clipboard::default(),
            )])),
        }
    }
}

pub async fn watch_config(path: String, config: Arc<Mutex<Config>>) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )
    .expect("Failed to create watcher")
    .watch(Path::new(&path), RecursiveMode::NonRecursive)
    .expect("Failed to watch file");

    while let Ok(event) = rx.recv() {
        if let EventKind::Modify(ModifyKind::Data(_)) = event.kind {
            debug!("Config file changed, reloading...");

            let new_config = Config::from_file(Path::new(&path))
                .await
                .map_err(|e| anyhow!("Found malformed config format. {e}"))?;
            let mut config_guard = config.lock().unwrap();

            *config_guard = new_config;

            debug!("Config updated: {:?}", *config_guard);
        }
    }

    Ok(())
}

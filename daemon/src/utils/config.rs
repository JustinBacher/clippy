use std::{
    collections::HashMap,
    path::Path,
    sync::{mpsc::channel, Arc},
};

use anyhow::{anyhow, Result};
use log::debug;
use notify::{event::ModifyKind, recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};

use super::get_cache_path;
use crate::database::clipboard::ClipEntry;

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

impl Clude {
    fn check_applications(&self, clip: &ClipEntry) -> Result<bool> {
        if let (Some(title), Some(applications)) = (&clip.application, &self.applications) {
            Ok(Regex::new(applications)?.is_match(title))
        } else {
            Ok(true)
        }
    }

    fn check_pattern(self, clip: &ClipEntry) -> Result<bool> {
        if let (Ok(ref payload), Some(patterns)) = (clip.text(), self.patterns) {
            Ok(Regex::new(&patterns)?.is_match(payload))
        } else {
            Ok(true)
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct General {
    pub polling_rate: Option<u64>,
    pub preview: Option<Preview>,
}

impl Default for General {
    fn default() -> Self {
        Self {
            polling_rate: Some(100),
            preview: Some(Preview::default()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Clipboard {
    pub db_path: String,
    pub max_size: Option<u64>,
    pub keep_duplicates: Option<u64>,
    pub remove_duplicates: Option<u64>,
    pub exclude: Option<Clude>,
    pub include: Option<Clude>,
}

impl Clipboard {
    pub fn can_store(self, clip: &ClipEntry) -> Result<bool> {
        for check in [self.exclude, self.include] {
            if let Some(clude) = check.as_ref() {
                return clude.check_applications(clip).and_then(|app_check| {
                    if app_check {
                        Ok(app_check)
                    } else {
                        clude.clone().check_pattern(clip)
                    }
                });
            }
        }
        Ok(true)
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            db_path: get_cache_path("db").unwrap().to_string_lossy().to_string(),
            max_size: Some(1_000),
            keep_duplicates: Some(10),
            remove_duplicates: Some(0),
            exclude: Some(Clude::default()),
            include: Some(Clude::default()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub general: Option<General>,
    pub clipboard: HashMap<String, Clipboard>,
}

impl Config {
    pub async fn from_file(path: &Path) -> Result<Self> {
        let config: Self = fs::read_to_string(path)
            .await
            .map_err(|err| anyhow!("Failed to read config file: {err}"))
            .and_then(|content| {
                toml::from_str(&content).map_err(|err| anyhow!("Failed to parse TOML: {err}"))
            })
            .unwrap();
        for board in config.clipboard.values() {
            let b = board.clone();
            if let (Some(_), Some(_)) = (b.include, b.exclude) {
                return Err(anyhow!("Config cannot have both include and exclude"));
            }
            if let (Some(_), Some(_)) = (board.keep_duplicates, board.remove_duplicates) {
                return Err(anyhow!(
                    "Config cannot have both keep_duplicates and remove_duplicates"
                ));
            }
        }
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: Some(General::default()),
            clipboard: HashMap::from([("default".to_string(), Clipboard::default())]),
        }
    }
}

pub async fn watch_config(path: String, config: Arc<Mutex<Config>>) -> Result<()> {
    let (tx, rx) = channel::<notify::Result<Event>>();

    recommended_watcher(tx)
        .expect("Failed to create watcher")
        .watch(Path::new(&path), RecursiveMode::NonRecursive)
        .expect("Failed to watch file");

    while let Ok(event) = rx.recv() {
        if let EventKind::Modify(ModifyKind::Data(_)) = event?.kind {
            debug!("Config file changed, reloading...");

            let new_config = Config::from_file(Path::new(&path))
                .await
                .map_err(|e| anyhow!("Found malformed config format. {e}"))?;
            let mut config_guard = config.lock().await;

            *config_guard = new_config;

            debug!("Config updated: {:?}", *config_guard);
        }
    }

    Ok(())
}

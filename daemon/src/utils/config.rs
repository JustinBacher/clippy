use serde::Deserialize;
use std::path::Path;
use tokio::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigStruct {
    name: String,
    version: String,
}

impl ConfigStruct {
    pub async fn from_file(path: &Path) -> Self {
        let content = fs::read_to_string(path).await.expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse config file")
    }
}

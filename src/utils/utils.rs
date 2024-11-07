use crate::prelude::Result;
use dirs::cache_dir;
use std::path::Path;

pub fn get_config_path() -> Result<String> {
    Ok(Path::join(&cache_dir().unwrap(), "/clippy/db")
        .to_str()
        .unwrap()
        .to_string())
}

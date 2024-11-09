use dirs::{cache_dir, config_dir};
use std::fs::create_dir_all;
use std::path::Path;

pub fn get_cache_path(path: &str, name: &str) -> Option<String> {
    let parent = Path::join(cache_dir()?.as_path(), Path::new(path));

    create_dir_all(parent.to_str()?).unwrap();

    Some(Path::join(&parent, Path::new(name)).to_str()?.to_string())
}

pub fn get_config_path(path: &str, name: &str) -> Option<String> {
    let parent = Path::join(config_dir()?.as_path(), Path::new(path));

    create_dir_all(parent.to_str()?).unwrap();

    Some(Path::join(&parent, Path::new(name)).to_str()?.to_string())
}

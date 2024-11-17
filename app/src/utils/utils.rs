use std::{fs::create_dir_all, path::Path};

use dirs::{cache_dir, config_dir};

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

#[cfg(test)]
use rand::{distributions::Alphanumeric, Rng};

#[cfg(test)]
pub fn random_str(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use dirs::{cache_dir, config_dir};

fn get_path(base_path: Option<PathBuf>, path: &str, name: &str) -> Option<String> {
    let parent = Path::join(base_path?.as_path(), Path::new(path));
    create_dir_all(parent.to_str()?).unwrap();
    Some(Path::join(&parent, Path::new(name)).to_str()?.to_string())
}

pub fn get_config_path(path: &str, name: &str) -> Option<String> {
    get_path(config_dir(), path, name)
}

pub fn get_cache_path(path: &str, name: &str) -> Option<String> {
    get_path(cache_dir(), path, name)
}

use rand::{distributions::Alphanumeric, Rng};

pub fn random_str(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

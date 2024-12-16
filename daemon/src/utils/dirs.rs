use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "", "clippy")
}

fn get_path<P>(base: &Path, path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let _ = create_dir_all(base);
    let full_path = base.join(path);
    Ok(full_path)
}

pub fn get_config_path<P>(name: &P) -> Result<PathBuf>
where
    P: AsRef<Path> + ?Sized,
{
    if let Some(project_dir) = get_project_dirs() {
        get_path(project_dir.config_dir(), name)
    } else {
        Err(anyhow!(""))
    }
}

pub fn get_cache_path<P>(name: &P) -> Result<PathBuf>
where
    P: AsRef<Path> + ?Sized,
{
    if let Some(project_dir) = get_project_dirs() {
        get_path(project_dir.cache_dir(), name)
    } else {
        Err(anyhow!(""))
    }
}

pub fn get_data_path<P>(name: &P) -> Result<PathBuf>
where
    P: AsRef<Path> + ?Sized,
{
    if let Some(project_dir) = get_project_dirs() {
        get_path(project_dir.data_dir(), name)
    } else {
        Err(anyhow!(""))
    }
}

use rand::{distributions::Alphanumeric, Rng};

pub fn random_str(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

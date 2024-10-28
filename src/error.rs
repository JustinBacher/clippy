use anyhow;
use redb;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    DatabaseError(#[from] redb::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

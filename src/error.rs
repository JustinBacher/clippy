use anyhow;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror;

enum DBError {
    CommitError,
    DatabaseError,
    StorageError,
    TableError,
    TransactionError,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Database Error")]
    DB(#[from] DBError),

    #[error("Unexpected error occured")]
    Unknown,
}

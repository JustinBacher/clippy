use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Error occured while committing to database")]
    Commit(#[from] CommitError),

    #[error("Database error occured")]
    DB(#[from] DatabaseError),

    #[error("Database error occured")]
    DBIO(#[from] StorageError),

    #[error("Database error occured")]
    TB(#[from] TableError),

    #[error("Database error occured")]
    DBTrans(#[from] TransactionError),

    #[allow(dead_code)]
    #[error("Unexpected error occured")]
    Unknown,
}

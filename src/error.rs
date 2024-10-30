use anyhow;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Database Error")]
    DB(#[from] DatabaseError),

    #[error("Error during database transation")]
    TX(#[from] TransactionError),

    #[error("Database Error")]
    TBL(#[from] TableError),

    #[error("Database Error")]
    CM(#[from] CommitError),

    #[error("Database Error")]
    ST(#[from] StorageError),

    #[error("Error Occured")]
    Other(#[from] anyhow::Error),
}

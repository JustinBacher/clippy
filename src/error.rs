use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("Error occured while committing to database")]
    Commit(#[from] CommitError),

    #[error("Database error occured")]
    DB(#[from] DatabaseError),

    #[error("Database error occured")]
    Storage(#[from] StorageError),

    #[error("Database error occured")]
    Table(#[from] TableError),

    #[error("Database error occured")]
    Transaction(#[from] TransactionError),
}

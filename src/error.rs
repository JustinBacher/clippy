use image::error::ImageError;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
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

    #[error("Error while collecting image data from history")]
    Decoding(#[from] ImageError),
}

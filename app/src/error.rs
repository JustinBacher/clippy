use image::error::ImageError;
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),

    
    #[error("Error while collecting image data from history")]
    Decoding(#[from] ImageError),

    #[error("Error while parsing value")]
    Parsing(&'static str),
}

pub use crate::error::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[allow(dead_code)]
pub struct W<T>(pub T);

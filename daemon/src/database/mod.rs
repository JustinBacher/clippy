pub mod clipboard;
pub mod node;
pub mod testing;

use std::cmp::Ord;

use bincode;
use derive_more::derive::Display;
use native_db::{Key, ToKey};
use serde::{Deserialize, Serialize};

struct Bincode;

impl<T: Serialize> native_model::Encode<T> for Bincode {
    type Error = bincode::error::EncodeError;

    fn encode(obj: &T) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::serde::encode_to_vec(obj, bincode::config::standard())
    }
}

impl<T: for<'a> Deserialize<'a>> native_model::Decode<T> for Bincode {
    type Error = bincode::error::DecodeError;

    fn decode(data: Vec<u8>) -> Result<T, bincode::error::DecodeError> {
        Ok(bincode::serde::decode_from_slice(&data, bincode::config::standard())?.0)
    }
}

#[derive(
    Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash, Copy, PartialOrd, Ord, Display,
)]
pub struct DateTime(pub chrono::DateTime<chrono::Local>);

impl DateTime {
    pub fn now() -> Self {
        Self(chrono::Local::now())
    }
}

impl From<chrono::DateTime<chrono::Local>> for DateTime {
    fn from(date: chrono::DateTime<chrono::Local>) -> Self {
        Self(date)
    }
}

impl ToKey for DateTime {
    fn to_key(&self) -> Key {
        Key::new(
            self.0
                .timestamp_nanos_opt()
                .unwrap_or_else(|| self.0.timestamp_micros())
                .to_be_bytes()
                .to_vec(),
        )
    }

    fn key_names() -> Vec<String> {
        vec!["DateTime".to_string()]
    }
}

use bincode;
pub use native_db::*;
use once_cell::sync::Lazy;
pub use schemas::ClipEntry;
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

pub mod schemas {
    use anyhow::Result;
    use native_model::{native_model, Model};

    use super::*;
    use crate::utils::detection::get_active_window;

    pub type ClipEntry = v1::ClipEntry;

    pub(super) mod v1 {
        use super::*;

        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash, Copy)]
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

        #[native_db]
        #[native_model(id = 1, version = 1, with = Bincode)]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
        pub struct ClipEntry {
            #[primary_key]
            pub epoch: DateTime,
            pub payload: Vec<u8>,
            pub application: Option<String>,
        }

        impl ClipEntry {
            pub fn new(payload: &[u8]) -> Self {
                Self {
                    epoch: v1::DateTime::now(),
                    payload: payload.to_vec(),
                    application: get_active_window(),
                }
            }

            pub fn text(&self) -> Result<String> {
                let str_ified = std::str::from_utf8(&self.payload)?;

                Ok(str_ified.to_string())
            }

            pub fn contains(&self, maybe_query: &Option<String>) -> bool {
                if let Some(query) = maybe_query {
                    if self.text().is_ok_and(|text| text.contains(query)) {
                        return true;
                    }
                }

                false
            }

            pub fn was_copied_from_app(&self, maybe_title: &Option<String>) -> bool {
                if let Some(title) = maybe_title {
                    if self.application.as_deref().unwrap_or_default().contains(title) {
                        return true;
                    }
                }

                false
            }
        }
    }
}

pub static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<schemas::v1::ClipEntry>().unwrap();
    models
});

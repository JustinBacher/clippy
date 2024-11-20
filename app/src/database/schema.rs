pub use native_db::*;
use once_cell::sync::Lazy;
pub use schemas::ClipEntry;

pub mod schemas {
    use native_db::{native_db, Key, ToKey};
    use native_model::{native_model, Model};
    use serde::{Deserialize, Serialize};

    pub type ClipEntry = v1::ClipEntry;

    pub(super) mod v1 {
        use super::*;

        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash, Copy)]
        pub struct DateTime(chrono::DateTime<chrono::Local>);

        impl DateTime {
            pub fn now() -> Self {
                Self(chrono::Local::now())
            }

            pub fn into_inner(self) -> chrono::DateTime<chrono::Local> {
                self.0
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
        #[native_model(id = 1, version = 1)]
        #[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone)]
        pub struct ClipEntry {
            #[primary_key]
            pub epoch: DateTime,
            pub payload: String,
        }
    }

    impl ClipEntry {
        pub fn new(payload: &str) -> Self {
            Self {
                epoch: v1::DateTime::now(),
                payload: payload.to_string(),
            }
        }
    }
}

pub static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<schemas::v1::ClipEntry>().unwrap();
    models
});

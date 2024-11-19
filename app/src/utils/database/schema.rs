mod schemas {
    use native_db::{native_db, primary_key};
    use native_model::{native_model, Model};
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};

    pub type ClipEntry = v1::ClipEntry;

    pub mod v1 {
        use super::*;
            
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Hash)]
        struct DateTime(chrono::DateTime<chrono::Local>);

        impl DateTime {
            fn now() -> Self {
                Self(chrono::Local::now())
            }
        }

        impl ToKey for DateTime {
            fn to_key(&self) -> Key {
                Key::new(self.0.timestamp_nanos().to_be_bytes().to_vec())
            }
        
            fn key_names() -> Vec<String> {
                vec!["DateTime".to_string()]
            }
        }

        #[native_db]
        #[native_model(id = 1, version = 1)]
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        pub struct ClipEntry {
            #[primary_key]
            epoch: DateTime,
            payload: String,
        }
    }
}
pub static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    // It's a good practice to define the models by specifying the version
    models.define::<schemas::v1::ClipEntry>().unwrap();
    models
 });
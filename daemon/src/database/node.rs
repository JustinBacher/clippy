#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;

use std::fs;

use anyhow::{anyhow, Result};
use blake3::Hasher;
use mac_address::get_mac_address;
pub use native_db::{
    native_db,
    transaction::{RTransaction, RwTransaction},
    Builder as DatabaseBuilder, Database, Key, KeyAttributes, Models, ToInput, ToKey,
};
use once_cell::sync::Lazy;
use whoami;

use super::*;
pub use schemas::Node;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DeviceIdentifier(String);

impl DeviceIdentifier {
    pub fn new() -> Result<Self> {
        let id = Self::get_machine_id()
            .or_else(|_| Self::get_mac_address())
            .or_else(|_| Self::generate_fallback_id())?;

        Ok(DeviceIdentifier(id))
    }

    fn get_machine_id() -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            fs::read_to_string("/etc/machine-id")
                .map(|id| id.trim().to_string())
                .map_err(|_| anyhow!("Could not read machine-id"))
        }

        #[cfg(target_os = "windows")]
        {
            // Windows Machine ID via WMI
            let output = Command::new("wmic")
                .args(&["csproduct", "get", "uuid"])
                .output()
                .map_err(|_| anyhow!("Failed to get Windows UUID"))?;

            String::from_utf8(output.stdout)
                .map(|id| id.lines().nth(1).unwrap_or("").trim().to_string())
                .map_err(|_| anyhow!("Invalid UUID"))
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("ioreg")
                .args(&["-d2", "-c", "IOPlatformExpertDevice"])
                .output()
                .map_err(|_| anyhow!("Failed to get macOS UUID"))?;

            let uuid_str = String::from_utf8(output.stdout).unwrap_or_default();

            let uuid = uuid_str
                .lines()
                .find(|line| line.contains("UUID"))
                .and_then(|line| line.split('"').nth(3).map(|s| s.to_string()))
                .ok_or(anyhow!("No UUID found"))?;

            Ok(uuid)
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(anyhow!("Unsupported OS"))
        }
    }

    fn get_mac_address() -> Result<String> {
        get_mac_address()
            .map_err(|_| "Could not get MAC address".to_string())
            .and_then(|mac| {
                if let Some(m) = mac {
                    Ok(m.to_string())
                } else {
                    Err(String::default())
                }
            })
            .map_err(|_| anyhow!("Invalid MAC"))
    }

    fn generate_fallback_id() -> Result<String> {
        let hostname = whoami::fallible::hostname().or_else(|_| whoami::fallible::devicename())?;
        let username = whoami::username();

        let hash = Hasher::new()
            .update(&hostname.into_bytes())
            .update(&username.into_bytes())
            .finalize()
            .to_string();

        Ok(hash)
    }
}

impl ToKey for DeviceIdentifier {
    fn to_key(&self) -> Key {
        Key::new(self.0.clone().into_bytes())
    }

    fn key_names() -> Vec<String> {
        vec!["DeviceIdentifier".to_string()]
    }
}
pub mod schemas {
    use native_db::native_db;
    use native_model::native_model;
    use serde::{Deserialize, Serialize};
    use std::net::IpAddr;

    use super::*;
    use crate::utils::ip::{get_local_ip, get_public_ip};

    pub type Node = v1::NodeV1;

    pub mod v1 {
        use super::*;
        use native_model::Model;

        #[native_db]
        #[native_model(id = 1, version = 1, with = Bincode)]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
        pub struct NodeV1 {
            #[primary_key]
            pub device_id: DeviceIdentifier,
            pub name: String,
            pub local_ip: IpAddr,
            pub public_ip: IpAddr,
            last_seen: Option<DateTime>,
        }

        impl NodeV1 {
            pub fn new() -> Self {
                Self {
                    device_id: DeviceIdentifier::new().unwrap(),
                    name: whoami::fallible::hostname().unwrap(),
                    local_ip: get_local_ip().unwrap(),
                    public_ip: get_public_ip().unwrap(),
                    last_seen: None,
                }
            }
            pub fn is_self(&self) -> Result<bool> {
                Ok(self.local_ip == get_local_ip()? && self.public_ip == get_public_ip()?)
            }
        }
    }
}

pub static NODE_MODEL: Lazy<Models> = Lazy::new(|| {
    let mut model = Models::new();
    model.define::<Node>().unwrap();
    model
});

#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;

use std::fs;

use anyhow::{Result, anyhow};
use blake3::Hasher;
use derive_more::derive::Display;
use itertools::Itertools;
use mac_address::get_mac_address;
pub use native_db::{
    Builder as DatabaseBuilder, Database, Key, KeyAttributes, Models, ToInput, ToKey, native_db,
    transaction::{RTransaction, RwTransaction},
};
use once_cell::sync::Lazy;
use rmp_serde::Serializer;
use std::net::IpAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use whoami;

use super::*;
use crate::{
    database::clipboard::ClipEntry,
    prelude::DEFAULT_PORTS,
    sync::utils::get_db,
    utils::ip::{get_local_ip, get_public_ip},
};
pub use schemas::Node;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DeviceIdentifier(String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Display)]
pub enum IpOrigin {
    Local,
    Public,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum NodeMessage {
    NewClip(ClipEntry),
    SyncRequest(Node),
    SyncResponse(Vec<ClipEntry>),
    JoinNetwork(Node),
}

impl DeviceIdentifier {
    pub fn new() -> Option<Self> {
        match Self::get_machine_id()
            .or_else(|_| Self::get_mac_address())
            .or_else(|_| Self::generate_fallback_id())
        {
            Ok(id) => Some(Self(id)),
            Err(e) => {
                eprintln!("Could not generate device ID: {e}");
                None
            },
        }
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
            .map_err(|_| anyhow!("Could not get MAC address"))
            .and_then(|mac| {
                if let Some(m) = mac {
                    return Ok(m.to_string());
                };
                Err(anyhow!("Malformed MAC address"))
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

    use super::*;
    pub type Node = v1::NodeV1;

    mod v1 {
        use super::*;
        use native_model::Model;

        #[native_db]
        #[native_model(id = 1, version = 1, with = Bincode)]
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash, Clone)]
        pub struct NodeV1 {
            #[primary_key]
            pub device_id: Option<DeviceIdentifier>,
            pub name: String,
            pub local_ip: IpAddr,
            pub public_ip: IpAddr,
            pub last_seen: Option<DateTime>,
            pub last_sync: Option<DateTime>,
            pub last_ip: Option<IpOrigin>,
        }
    }
}

impl Node {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            device_id: Some(DeviceIdentifier::new().unwrap()),
            name: whoami::fallible::hostname().unwrap(),
            local_ip: get_local_ip().unwrap(),
            public_ip: get_public_ip().unwrap(),
            last_seen: None,
            last_sync: None,
            last_ip: None,
        }
    }

    pub fn is_self(&self) -> Result<bool> {
        Ok(self.local_ip == get_local_ip()? && self.public_ip == get_public_ip()?)
    }

    async fn attempt_connection(&self, ip: IpAddr) -> Result<TcpStream> {
        for port in DEFAULT_PORTS.iter() {
            if let Ok(stream) = TcpStream::connect((ip, *port)).await {
                return Ok(stream);
            }
        }
        Err(anyhow!(""))
    }

    pub async fn get_stream(&self) -> Result<TcpStream> {
        match self.last_ip {
            Some(IpOrigin::Public) => {
                if let Ok(stream) = self.attempt_connection(self.local_ip).await {
                    return Ok(stream);
                }
                if let Ok(stream) = self.attempt_connection(self.public_ip).await {
                    return Ok(stream);
                }
                Err(anyhow!(""))
            },
            Some(IpOrigin::Local) | None => {
                if let Ok(stream) = self.attempt_connection(self.local_ip).await {
                    return Ok(stream);
                }
                if let Ok(stream) = self.attempt_connection(self.public_ip).await {
                    return Ok(stream);
                }
                Err(anyhow!(""))
            },
        }
    }

    pub async fn send_clip(&self, clip: ClipEntry) -> Result<()> {
        let message = NodeMessage::NewClip(clip);

        let Ok(mut stream) = self.get_stream().await else {
            return Err(anyhow!(""));
        };
        let mut buffer = Vec::new();

        message.serialize(&mut Serializer::new(&mut buffer))?;
        stream.write_all(&(buffer.len() as u32).to_be_bytes()).await?;
        stream.write_all(&buffer).await?;

        Ok(())
    }

    pub async fn send_clips<'a>(&self, clips: Vec<ClipEntry>) -> Result<()> {
        let Ok(mut stream) = self.get_stream().await else {
            return Err(anyhow!(""));
        };
        let mut buffer = Vec::new();

        clips.serialize(&mut Serializer::new(&mut buffer))?;
        stream.write_all(&(buffer.len() as u32).to_be_bytes()).await?;
        stream.write_all(&buffer).await?;

        Ok(())
    }

    pub async fn sync(&self) -> Result<()> {
        let last_sync = self.last_sync;
        let db = get_db()?;
        let tx = db.r_transaction()?;

        let clips = tx
            .scan()
            .primary::<ClipEntry>()?
            .all()?
            .flatten()
            .filter(|entry| last_sync.is_some_and(|last_sync| entry.epoch > last_sync))
            .collect_vec();

        if let Err(e) = self.send_clips(clips).await {
            eprintln!("Unable to sync with node: {self:?}: {e}");
        }
        Ok(())
    }
}

pub static NODE_MODEL: Lazy<Models> = Lazy::new(|| {
    let mut model = Models::new();
    model.define::<Node>().unwrap();
    model
});

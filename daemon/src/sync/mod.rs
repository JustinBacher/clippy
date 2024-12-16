pub mod connection;
pub mod node_manager;

use std::{net::IpAddr, sync::Arc};

use anyhow::{Result, anyhow};
use tokio::net::TcpListener;

use crate::prelude::DEFAULT_PORTS;

async fn get_listener_on_available_port(ip: IpAddr) -> Result<Arc<TcpListener>> {
    for port in DEFAULT_PORTS {
        if let Ok(listener) = TcpListener::bind((ip, *port)).await {
            return Ok(Arc::new(listener));
        } else {
            continue;
        }
    }
    Err(anyhow!("Could not create a connection on {ip}"))
}

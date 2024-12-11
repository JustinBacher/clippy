use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use tokio::net::TcpListener;

pub mod connection;

async fn get_listener_on_available_port(ip: IpAddr) -> Result<Arc<TcpListener>> {
    for port in 53000..53100 {
        let socket = SocketAddr::new(ip, port);
        if let Ok(listener) = TcpListener::bind(socket).await {
            return Ok(Arc::new(listener));
        } else {
            continue;
        }
    }
    Err(anyhow!("Could not create a connection on {ip}"))
}

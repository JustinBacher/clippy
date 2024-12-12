use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpListener,
    sync::{mpsc, Mutex},
};

use super::utils::NodeManager;
use super::*;
use crate::{
    database::{clipboard::ClipEntry, node::Node},
    utils::{clipboard::respond_to_clip, config::Config},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum NodeMessage {
    Put(ClipEntry),
    Get(String),
    GetResponse(Option<ClipEntry>),
    JoinNetwork(Node),
}

async fn listen(stream: Arc<TcpListener>) {
    match stream.accept().await {
        Ok((socket, _)) => {
            tokio::spawn(async move {
                let mut buffer = Vec::new();

                if let Err(e) = BufReader::new(socket).read_buf(&mut buffer).await {
                    eprintln!("Error reading socket: {e}")
                } else {
                    println!("Received message: {:?}", String::from_utf8_lossy(&buffer))
                }
            });
        },
        Err(e) => eprintln!("Error accepting connection: {e}"),
    }
}

#[derive(Clone)]
pub struct DistributedHashNetwork<'a> {
    local_node: Node,
    node_manager: NodeManager<'a>,
    config: Arc<Mutex<Config>>,
    message_tx: mpsc::Sender<NodeMessage>,
}

impl<'a> DistributedHashNetwork<'a> {
    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let (message_tx, mut message_rx) = mpsc::channel(100);

        let local_node = Node::new();
        let node_manager = NodeManager::new()?;
        let manager = node_manager.clone();
        let conf = Arc::clone(&config);

        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                match message {
                    NodeMessage::Put(clip) => {
                        if let Err(e) = respond_to_clip(&conf, clip).await {
                            eprintln!("Clip entry error: {e}");
                        }
                    },
                    NodeMessage::Get(_key) => {
                        unimplemented!();
                    },
                    NodeMessage::JoinNetwork(node) => {
                        if let Err(e) = manager.clone().join(node) {
                            eprintln!("Unable to perform handshake with device: {e}")
                        }
                    },
                    _ => {},
                }
            }
        });

        Ok(Self {
            local_node,
            node_manager,
            config,
            message_tx,
        })
    }

    pub async fn put(&self, clip: ClipEntry) -> Result<()> {
        self.message_tx
            .send(NodeMessage::Put(clip))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;
        Ok(())
    }

    // Get a value from the distributed hash table
    pub async fn get(&self, _key: String) -> Result<Option<String>> {
        // First check local store

        // If not found locally, send get message
        self.message_tx
            .send(NodeMessage::Get(_key))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;

        // In a real implementation, you'd wait for a response
        Ok(None)
    }

    // Start the node's server to listen for incoming connections
    pub async fn start_server(&self) -> Result<()> {
        let listener = get_listener_on_available_port(self.local_node.local_ip).await?;

        tokio::spawn({
            async move {
                loop {
                    listen(listener.clone()).await;
                }
            }
        });

        Ok(())
    }

    // Join the network by adding a known node
    pub async fn join_network(&self, node: Node) -> Result<()> {
        self.message_tx
            .send(NodeMessage::JoinNetwork(node))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;
        Ok(())
    }
}

// impl Clone for DistributedHashNetwork {
//     fn clone(&self) -> Self {
//         Self {
//             local_node: self.local_node.clone(),
//             known_nodes: Arc::clone(&self.known_nodes),
//             message_tx: self.message_tx.clone(),
//         }
//     }
// }

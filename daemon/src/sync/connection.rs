use std::io::Write;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
    sync::{mpsc, Mutex},
};

use super::utils::NodeManager;
use super::*;
use crate::{
    database::{clipboard::ClipEntry, node::Node},
    utils::{clipboard::respond_to_clip, config::Config, utils::get_cache_path},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeMessage {
    Put(ClipEntry),
    GetResponse(Option<ClipEntry>),
    JoinNetwork(Node),
}

#[derive(Clone)]
pub struct DistributedHashNetwork {
    local_node: Node,
    node_manager: NodeManager,
    config: Arc<Mutex<Config>>,
    message_tx: mpsc::Sender<NodeMessage>,
    message_rx: Arc<Mutex<mpsc::Receiver<NodeMessage>>>,
}

impl DistributedHashNetwork {
    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let (message_tx, message_rx) = mpsc::channel(100);

        let dhn = Self {
            local_node: Node::new(),
            node_manager: NodeManager::new()?,
            config: config.clone(),
            message_tx: message_tx.clone(),
            message_rx: Arc::new(Mutex::new(message_rx)),
        };

        let responder = Arc::new(Mutex::new(dhn.clone()));
        tokio::spawn(async move {
            let this = responder.lock().await;
            let mut rx = this.message_rx.lock().await;
            while let Some(message) = rx.recv().await {
                match message {
                    NodeMessage::Put(clip) => {
                        respond_to_clip(&config, clip).await.unwrap();
                    },
                    NodeMessage::GetResponse(value) => {
                        // Handle get response (could be used to implement more complex routing)
                        if let Some(v) = value {
                            println!("Received value: {:?}", v);
                        }
                    },
                    NodeMessage::JoinNetwork(node) => {
                        this.node_manager.join(node).unwrap();
                    },
                }
            }
        });

        Ok(dhn)
    }

    pub async fn put(&self, clip: ClipEntry) -> Result<()> {
        self.message_tx
            .send(NodeMessage::Put(clip))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;
        Ok(())
    }

    pub async fn start_server(&self) -> Result<()> {
        let listener = get_listener_on_available_port(self.local_node.local_ip).await?;

        let this = self.clone();
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            loop {
                match listener.accept().await {
                    Ok((socket, _)) => {
                        let mut reader = BufReader::new(socket);

                        let mut len_buffer = [0u8; 4];
                        reader.read_exact(&mut len_buffer).await.unwrap();
                        let message_len = u32::from_le_bytes(len_buffer) as usize;

                        {
                            buffer.resize(message_len, 0u8);
                            reader.read_exact(&mut buffer).await.unwrap();
                        }
                        let b = &mut buffer.as_slice();
                        let mut buf = Deserializer::new(b);

                        match NodeMessage::deserialize(&mut buf) {
                            Ok(message) => match this.message_tx.send(message).await {
                                Ok(_) => println!("Message processed successfully"),
                                Err(e) => eprintln!("Failed to process message: {e}"),
                            },
                            Err(e) => eprintln!("Failed to deserialize message: {e}"),
                        }
                        buffer.clear();
                    },
                    Err(e) => eprintln!("Error accepting connection: {e}"),
                }
            }
        });

        Ok(())
    }

    pub async fn send_clip(&self, clip: ClipEntry) -> Result<()> {
        let message = NodeMessage::Put(clip);

        for node in self.node_manager.get_nodes()?.iter() {
            let Ok(mut stream) = node.get_stream().await else {
                return Err(anyhow!(""));
            };
            let mut buffer = Vec::new();

            message.serialize(&mut Serializer::new(&mut buffer))?;
            stream.write_all(&buffer).await?;
        }

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

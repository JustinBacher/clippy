use std::sync::Arc;

use anyhow::Result;
use rmp_serde::Deserializer;
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, BufReader},
    sync::{Mutex, RwLock, mpsc},
    time::{Duration, sleep},
};

use super::{node_manager::NodeManager, *};
use crate::{
    database::node::{Node, NodeMessage},
    utils::{clipboard::respond_to_clip, config::Config},
};

#[derive(Clone)]
pub struct DistributedHashNetwork {
    local_node: Node,
    node_manager: NodeManager,
    #[allow(dead_code)]
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
            config: Arc::clone(&config),
            message_tx: message_tx.clone(),
            message_rx: Arc::new(Mutex::new(message_rx)),
        };

        Ok(dhn)
    }

    pub async fn start_server(self) -> Result<()> {
        let listener = get_listener_on_available_port(self.local_node.local_ip).await?;

        for node in self.node_manager.get_nodes()?.into_iter() {
            node.send_message(NodeMessage::JoinNetwork(node.clone())).await?;
        }

        // I'll be honest this is ugly but it somehow circumvents lifetime bullshit
        let dhn = Arc::new(RwLock::new(Box::leak(Box::new(self))));
        let receiver = Arc::clone(&dhn);

        tokio::spawn(async move {
            let this = receiver.read().await;
            let mut rx = this.message_rx.lock().await;
            loop {
                while let Some(message) = rx.recv().await {
                    match message {
                        NodeMessage::NewClip(ref clip) => {
                            respond_to_clip(&this.config, clip).await.unwrap()
                        },
                        NodeMessage::SyncRequest(node) => {
                            if let Err(e) = node.sync().await {
                                eprintln!("Unable to sync with {node:?}: {e}")
                            }
                        },
                        NodeMessage::SyncResponse(clips) => {
                            println!("Received clips: {clips:?}");
                            for clip in clips.iter() {
                                respond_to_clip(&this.config, clip).await.unwrap();
                            }
                        },
                        NodeMessage::JoinNetwork(node) => {
                            this.node_manager.add_node(&node).unwrap();
                        },
                    }
                }
                sleep(Duration::from_secs(60 * 5)).await;
            }
        });

        tokio::spawn(async move {
            let this = dhn.write().await;
            loop {
                loop {
                    let socket = match listener.accept().await {
                        Ok((s, _)) => s,
                        Err(e) => {
                            eprintln!("Error accepting connection: {e}");
                            break;
                        },
                    };

                    let mut reader = BufReader::new(socket);
                    let mut len_bytes = [0u8; 4];
                    reader.read_exact(&mut len_bytes).await.unwrap();
                    let len = u32::from_be_bytes(len_bytes) as usize;

                    let mut buffer = vec![0u8; len];
                    reader.read_exact(&mut buffer).await.unwrap();

                    match NodeMessage::deserialize(&mut Deserializer::new(&*buffer)) {
                        Ok(message) => {
                            if let Err(e) = this.message_tx.send(message).await {
                                eprint!("Failed to process message: {e}.");
                            }
                        },
                        Err(e) => eprint!("Failed to deserialize message: {e}."),
                    }
                }
                sleep(Duration::from_secs(60 * 5)).await;
            }
        });
        Ok(())
    }
}

use std::sync::Arc;

use anyhow::anyhow;
use rmp_serde::Deserializer;
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, BufReader},
    sync::{Mutex, mpsc},
    time::{Duration, sleep},
};

use super::{utils::NodeManager, *};
use crate::{
    database::{
        clipboard::ClipEntry,
        node::{Node, NodeMessage},
    },
    utils::{clipboard::respond_to_clip, config::Config},
};

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
            config: Arc::clone(&config),
            message_tx: message_tx.clone(),
            message_rx: Arc::new(Mutex::new(message_rx)),
        };

        let responder = Arc::new(Mutex::new(dhn.clone()));
        tokio::spawn(async move {
            let this = responder.lock().await;
            let mut rx = this.message_rx.lock().await;
            loop {
                while let Some(message) = rx.recv().await {
                    match message {
                        NodeMessage::NewClip(ref clip) => {
                            respond_to_clip(&config, clip).await.unwrap()
                        },
                        NodeMessage::SyncRequest(node) => {
                            if let Err(e) = node.sync().await {
                                eprintln!("Unable to sync with {node:?}: {e}")
                            }
                        },
                        NodeMessage::SyncResponse(clips) => {
                            println!("Received clips: {clips:?}",);
                            for clip in clips.iter() {
                                respond_to_clip(&config, clip).await.unwrap();
                            }
                        },
                        NodeMessage::JoinNetwork(node) => {
                            this.node_manager.join(node).unwrap();
                        },
                    }
                }
                sleep(Duration::from_secs(60 * 5)).await;
            }
        });

        Ok(dhn)
    }

    pub async fn send_clip(&self, clip: ClipEntry) -> Result<()> {
        self.message_tx
            .send(NodeMessage::NewClip(clip))
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
                loop {
                    let mut reader = match listener.accept().await {
                        Ok((s, _)) => BufReader::new(s),
                        Err(e) => {
                            eprintln!("Error accepting connection: {e}");
                            break;
                        },
                    };
                    reader.read_exact(&mut buffer).await.unwrap();

                    match NodeMessage::deserialize(&mut Deserializer::new(&*buffer)) {
                        Ok(message) => match this.message_tx.send(message).await {
                            Ok(_) => println!("Message processed successfully"),
                            Err(e) => eprint!("Failed to process message: {e}."),
                        },
                        Err(e) => eprint!("Failed to deserialize message: {e}."),
                    }
                    buffer.clear();
                }
                sleep(Duration::from_secs(60 * 5)).await;
            }
        });
        Ok(())
    }

    pub async fn join_network(&self, node: Node) -> Result<()> {
        self.message_tx
            .send(NodeMessage::JoinNetwork(node))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;
        Ok(())
    }
}

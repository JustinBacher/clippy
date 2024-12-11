use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use camino::Utf8Path;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use native_db::{Builder as DatabaseBuilder, Database};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};

use super::*;
use crate::{
    database::{
        clipboard::{get_db, schemas::ClipEntry},
        node::{Node, NODE_MODEL},
    },
    utils::{clipboard::respond_to_clip, config::Config, get_cache_path},
};

struct NodeManager<'a> {
    db: Arc<Database<'a>>,
}

impl<'a> NodeManager<'a> {
    pub fn new() -> Result<Self> {
        let db_path = get_cache_path(&Path::new("util").join("db")).unwrap();
        let database = DatabaseBuilder::new()
            .create(&NODE_MODEL, db_path)
            .map_err(|_| anyhow!("Could not create peer database."))?;

        let manager = Self {
            db: Arc::new(database),
        };

        Ok(manager)
    }

    pub fn get_nodes(&self) -> Result<Vec<Node>> {
        let tx = self.db.r_transaction()?;
        Ok(tx.scan().primary::<Node>()?.all()?.flatten().collect_vec())
    }
}

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

struct DistributedHashNetwork<'a> {
    local_node: Node,
    node_manager: NodeManager<'a>,
    config: Arc<Mutex<Config>>,
    message_tx: mpsc::Sender<NodeMessage>,
}

impl<'a> DistributedHashNetwork<'a> {
    async fn new(config: Arc<std::sync::Mutex<Config>>) -> Result<Self> {
        let (message_tx, mut message_rx) = mpsc::channel(100);

        let local_node = Node::new();
        let node_manager = NodeManager::new()?;

        tokio::spawn(async move {
            while let Some(message) = message_rx.recv().await {
                match message {
                    NodeMessage::Put(clip) => {
                        if let Err(e) = respond_to_clip(config, clip).await {
                            eprintln!("Clip entry error: {e}");
                        }
                    },
                    NodeMessage::Get(key) => {
                        let db = get_db(Utf8Path::new(
                            get_cache_path("primary").unwrap().as_path().to_str().unwrap(),
                        ))
                        .unwrap();

                        if let Ok(Some(value)) = db.r_transaction().unwrap().get().primary(key) {
                            message_tx
                                .send(NodeMessage::GetResponse(value))
                                .await
                                .map_err(|_| anyhow!("Failed to send put message"))
                                .unwrap();
                        }
                    },
                    NodeMessage::JoinNetwork(node) => {
                        let db = get_db(Utf8Path::new(
                            get_cache_path("primary").unwrap().as_path().to_str().unwrap(),
                        ))
                        .unwrap();

                        let known_nodes = db.r_transaction().and_then(|tx| {
                            tx.scan().primary::<Node>().and_then(|it| {
                                it.all().map_err(|_| "Invalid node".to_string()).and_then(
                                    |cursor| {
                                        cursor.filter(|n| {
                                            if n == node {
                                                return true;
                                            } else {
                                                return false;
                                            }
                                        })
                                    },
                                )
                            })
                        });
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

    async fn put(&self, key: String, value: ClipEntry) -> Result<()> {
        self.message_tx
            .send(NodeMessage::Put(key, value))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;
        Ok(())
    }

    // Get a value from the distributed hash table
    async fn get(&self, key: String) -> Result<Option<String>> {
        // First check local store
        todo!("Implement a way to retrieve a missing clip");

        // If not found locally, send get message
        self.message_tx
            .send(NodeMessage::Get(key))
            .await
            .map_err(|_| anyhow!("Failed to send put message"))?;

        // In a real implementation, you'd wait for a response
        Ok(None)
    }

    // Start the node's server to listen for incoming connections
    async fn start_server(&self) -> Result<()> {
        let listener = get_listener_on_available_port(self.local_node.local_ip).await?;

        tokio::spawn({
            async move {
                loop {
                    listen(listener.clone());
                }
            }
        });

        Ok(())
    }

    // Join the network by adding a known node
    async fn join_network(&self, node: Node) -> Result<()> {
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

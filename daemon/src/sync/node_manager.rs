use std::path::Path;

use anyhow::{Result, anyhow};
use itertools::Itertools;
use native_db::{Builder as DatabaseBuilder, Database};

use crate::{
    database::node::{NODE_MODEL, Node},
    utils::get_cache_path,
};

#[derive(Clone, Copy, Debug)]
pub struct NodeManager;

impl NodeManager {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn get_nodes(&self) -> Result<Vec<Node>> {
        let db = get_db()?;
        let tx = db.r_transaction()?;
        Ok(tx.scan().primary::<Node>()?.all()?.flatten().collect_vec())
    }

    pub fn add_node(&self, node: &Node) -> Result<()> {
        if !self.get_nodes()?.contains(node) {
            let db = get_db()?;
            let tx = db.rw_transaction()?;

            tx.insert(node.clone())?;
            tx.commit()?;
        }
        Ok(())
    }
}

pub fn get_db<'a>() -> Result<Database<'a>> {
    let db_path = get_cache_path(&Path::new("util").join("db")).unwrap();
    let db = DatabaseBuilder::new()
        .create(&NODE_MODEL, &db_path)
        .map_err(|e| anyhow!("Could not create peer database.{db_path:?}\n{e}"))?;

    Ok(db)
}

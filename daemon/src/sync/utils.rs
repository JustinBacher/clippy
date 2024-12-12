use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use native_db::{Builder as DatabaseBuilder, Database};

use crate::{
    database::node::{Node, NODE_MODEL},
    utils::get_cache_path,
};

#[derive(Clone)]
pub struct NodeManager<'a>(Arc<Database<'a>>);

impl<'a> NodeManager<'a> {
    pub fn new() -> Result<Self> {
        let db_path = get_cache_path(&Path::new("util").join("db")).unwrap();
        let database = DatabaseBuilder::new()
            .create(&NODE_MODEL, db_path.clone())
            .map_err(|e| anyhow!("Could not create peer database.{db_path:?}\n{e}"))?;

        let manager = Self(Arc::new(database));

        Ok(manager)
    }

    pub fn get_nodes(&self) -> Result<Vec<Node>> {
        let tx = self.0.r_transaction()?;
        Ok(tx.scan().primary::<Node>()?.all()?.flatten().collect_vec())
    }

    pub fn join(&self, node: Node) -> Result<()> {
        if !self.get_nodes()?.contains(&node) {
            let tx = self.0.rw_transaction()?;
            tx.insert(node)?;
            tx.commit()?;
        }
        Ok(())
    }
}

use std::{collections::HashMap, sync::Arc};

use super::TargetTcpClient;

pub struct TargetTcpConnections {
    connections: HashMap<u32, Arc<TargetTcpClient>>,
}

impl TargetTcpConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn add(&mut self, connection: Arc<TargetTcpClient>) {
        self.connections.insert(connection.id, connection);
    }
    pub fn get(&self, connection_id: u32) -> Option<&Arc<TargetTcpClient>> {
        self.connections.get(&connection_id)
    }

    pub fn remove(&mut self, connection_id: u32) -> Option<Arc<TargetTcpClient>> {
        self.connections.remove(&connection_id)
    }

    pub fn remove_all(self) -> HashMap<u32, Arc<TargetTcpClient>> {
        self.connections
    }
}

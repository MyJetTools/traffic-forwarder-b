use std::{collections::HashMap, sync::Arc};

use super::TcpClientToTarget;

pub struct TcpClientToTargetConnections {
    connections: HashMap<u32, Arc<TcpClientToTarget>>,
}

impl TcpClientToTargetConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    pub fn add(&mut self, connection: Arc<TcpClientToTarget>) {
        self.connections.insert(connection.id, connection);
    }
    pub fn get(&self, connection_id: u32) -> Option<&Arc<TcpClientToTarget>> {
        self.connections.get(&connection_id)
    }

    pub fn remove(&mut self, connection_id: u32) -> Option<Arc<TcpClientToTarget>> {
        self.connections.remove(&connection_id)
    }

    pub fn remove_all(self) -> HashMap<u32, Arc<TcpClientToTarget>> {
        self.connections
    }
}

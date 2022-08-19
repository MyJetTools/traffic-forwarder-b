use std::sync::Arc;

use my_tcp_sockets::tcp_connection::SocketConnection;
use traffic_forwarder_shared::tcp_tunnel::{TunnelTcpContract, TunnelTcpSerializer};

use crate::target_tcp_client::{TargetTcpClient, TargetTcpConnections};

pub struct TcpTunnel {
    tunnel_connection: Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>,
    target_connections: TargetTcpConnections,
}

impl TcpTunnel {
    pub fn new(
        tunnel_connection: Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>,
    ) -> Self {
        Self {
            tunnel_connection,
            target_connections: TargetTcpConnections::new(),
        }
    }

    pub fn dispose(self) {
        let connections = self.target_connections.remove_all();

        for connection in connections.values() {
            connection.disconnect();
        }
    }

    pub fn get_tunnel_connection(
        &self,
    ) -> Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>> {
        self.tunnel_connection.clone()
    }

    pub async fn send_payload_to_tunnel(&self, connection_id: u32, payload: Vec<u8>) {
        self.tunnel_connection
            .send(TunnelTcpContract::Payload {
                id: connection_id,
                payload,
            })
            .await
    }

    async fn send_disconnect_to_tunnel(&self, connection_id: u32) {
        self.tunnel_connection
            .send(TunnelTcpContract::Disconnected(connection_id))
            .await
    }

    pub fn add_target_connection(&mut self, tcp_client_to_target: Arc<TargetTcpClient>) {
        self.target_connections.add(tcp_client_to_target);
    }

    pub fn send_payload_to_target(&self, connection_id: u32, payload: Vec<u8>) {
        if let Some(target_connection) = self.target_connections.get(connection_id) {
            target_connection.send_payload(payload);
        }
    }

    pub async fn disconnect_target_connection(&mut self, connection_id: u32) {
        if let Some(removed_connection) = self.target_connections.remove(connection_id) {
            removed_connection.disconnect();
            self.send_disconnect_to_tunnel(connection_id).await;
        }
    }
}

use std::sync::Arc;

use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::Mutex;
use traffic_forwarder_shared::tcp_tunnel::{TunnelTcpContract, TunnelTcpSerializer};

use crate::target_tcp_client::TargetTcpClient;

use super::TcpTunnel;

pub struct TunnelTcpConnection {
    tcp_tunnel: Mutex<Option<TcpTunnel>>,
}

impl TunnelTcpConnection {
    pub fn new() -> Self {
        Self {
            tcp_tunnel: Mutex::new(None),
        }
    }

    pub async fn new_target_connection_established(
        &self,
        tcp_client_to_target: &Arc<TargetTcpClient>,
    ) -> Option<i32> {
        let connection_id = tcp_client_to_target.id;
        let mut write_access = self.tcp_tunnel.lock().await;

        if let Some(tunnel) = write_access.as_mut() {
            tunnel.add_target_connection(tcp_client_to_target.clone());
            self.send_connection_is_established_to_tunnel(connection_id)
                .await;
            Some(tunnel.get_tunnel_connection_id())
        } else {
            None
        }
    }

    async fn get_tunnel_connection(
        &self,
    ) -> Option<Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>> {
        let read_access = self.tcp_tunnel.lock().await;
        let tunnel = read_access.as_ref()?;
        Some(tunnel.get_tunnel_connection())
    }

    pub async fn send_can_not_establish_target_connection_to_tunnel(
        &self,
        connection_id: u32,
        err: String,
    ) {
        if let Some(tunnel_connection) = self.get_tunnel_connection().await {
            tunnel_connection
                .send(TunnelTcpContract::CanNotConnect {
                    id: connection_id,
                    reason: err,
                })
                .await;
        }
    }

    pub async fn send_connection_is_established_to_tunnel(&self, connection_id: u32) {
        let tunnel_access = self.tcp_tunnel.lock().await;

        if let Some(tunnel) = tunnel_access.as_ref() {
            tunnel
                .send_connection_is_established_to_tunnel(connection_id)
                .await;
        }
    }

    pub async fn disconnect_target_tcp_connection(&self, connection_id: u32) {
        let mut tunnel_access = self.tcp_tunnel.lock().await;
        if let Some(tunnel) = tunnel_access.as_mut() {
            tunnel.disconnect_target_connection(connection_id).await;
        }
    }

    pub async fn send_payload_to_target(&self, connection_id: u32, payload: Vec<u8>) {
        let tunnel_access = self.tcp_tunnel.lock().await;
        if let Some(tunnel) = tunnel_access.as_ref() {
            tunnel.send_payload_to_target(connection_id, payload);
        }
    }

    pub async fn tunnel_is_connected(
        &self,
        tunnel_connection: Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>,
    ) {
        let new_tunnel = TcpTunnel::new(tunnel_connection);

        let mut tunnel_access = self.tcp_tunnel.lock().await;

        if let Some(old_tunnel) = tunnel_access.replace(new_tunnel) {
            old_tunnel.dispose();
        }
    }

    pub async fn tunnel_is_disconnected(&self) {
        let mut tunnel_access = self.tcp_tunnel.lock().await;

        if let Some(old_tunnel) = tunnel_access.take() {
            old_tunnel.dispose();
        }
    }

    pub async fn send_payload_to_tunnel(&self, id: u32, payload: Vec<u8>) -> bool {
        let tunnel_access = self.tcp_tunnel.lock().await;

        if let Some(tunnel) = tunnel_access.as_ref() {
            tunnel.send_payload_to_tunnel(id, payload).await;
            true
        } else {
            false
        }
    }
}

use std::sync::Arc;

use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::Mutex;
use traffic_forwarder_shared::tcp_tunnel::{TunnelTcpContract, TunnelTcpSerializer};

use crate::target_tcp_client::TargetTcpClient;

use super::TcpTunnel;

pub enum DisconnectReason {
    DisconnectedFromSideA,
    DisconnectedFromSideB,
}

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
        target_tcp_client: &Arc<TargetTcpClient>,
    ) -> Option<Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>> {
        let connection = {
            let mut write_access = self.tcp_tunnel.lock().await;

            if let Some(tunnel) = write_access.as_mut() {
                tunnel.target_connections.add(target_tcp_client.clone());

                Some(tunnel.get_tunnel_connection())
            } else {
                None
            }
        };

        if let Some(connection) = connection.as_ref() {
            connection
                .send(TunnelTcpContract::Connected(target_tcp_client.id))
                .await;
        }

        connection
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

    pub async fn disconnect_target_tcp_connection(
        &self,
        connection_id: u32,
        reason: DisconnectReason,
    ) {
        let tunnel_connection = {
            let mut tunnel_access = self.tcp_tunnel.lock().await;
            if let Some(tunnel) = tunnel_access.as_mut() {
                if let Some(target_connection) = tunnel.target_connections.remove(connection_id) {
                    target_connection.disconnect();

                    if let DisconnectReason::DisconnectedFromSideB = reason {
                        Some(tunnel.get_tunnel_connection())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(tunnel_connection) = tunnel_connection {
            tunnel_connection
                .send(TunnelTcpContract::DisconnectedFromSideB(connection_id))
                .await;
        }
    }

    pub async fn send_payload_to_target(&self, connection_id: u32, payload: Vec<u8>) {
        let target_connection = {
            let tunnel_access = self.tcp_tunnel.lock().await;

            if let Some(tunnel_access) = tunnel_access.as_ref() {
                if let Some(target_connection) = tunnel_access.target_connections.get(connection_id)
                {
                    Some(target_connection.clone())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(target_connection) = target_connection {
            target_connection.send_payload(payload);
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

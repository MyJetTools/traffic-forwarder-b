use std::sync::Arc;

use my_tcp_sockets::tcp_connection::SocketConnection;
use traffic_forwarder_shared::tcp_tunnel::{TunnelTcpContract, TunnelTcpSerializer};

use crate::target_tcp_client::TargetTcpConnections;

pub struct TcpTunnel {
    tunnel_connection: Arc<SocketConnection<TunnelTcpContract, TunnelTcpSerializer>>,
    pub target_connections: TargetTcpConnections,
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
}

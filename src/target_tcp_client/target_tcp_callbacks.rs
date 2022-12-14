use std::sync::Arc;

use crate::app::AppContext;

use super::TargetTcpClient;

pub struct TargetTcpCallbacks {
    app: Arc<AppContext>,
}

impl TargetTcpCallbacks {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
    pub async fn on_disconnected(&self, tcp_client: Arc<TargetTcpClient>) {
        self.app
            .tunnel_tcp_connection
            .disconnect_target_tcp_connection(
                tcp_client.id,
                crate::tcp_tunnel::DisconnectReason::DisconnectedFromSideB,
            )
            .await;
    }

    pub async fn on_payload(&self, tcp_client: &Arc<TargetTcpClient>, payload: Vec<u8>) -> bool {
        self.app
            .tunnel_tcp_connection
            .send_payload_to_tunnel(tcp_client.id, payload)
            .await
    }
}

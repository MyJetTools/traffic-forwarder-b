use std::sync::Arc;

use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};

use crate::{
    app::AppContext,
    target_tcp_client::{TargetTcpCallbacks, TargetTcpClient},
};

use traffic_forwarder_shared::tcp_tunnel::{TunnelTcpContract, TunnelTcpSerializer};

pub struct TunnelTcpEvents {
    app: Arc<AppContext>,
}

impl TunnelTcpEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    async fn handle_payload(&self, payload: TunnelTcpContract) {
        match payload {
            TunnelTcpContract::Ping => {
                // N/A
            }
            TunnelTcpContract::Pong => {
                // N/A
            }
            TunnelTcpContract::ConnectTo {
                id,
                remote_host_port,
            } => {
                // A asks to connect to B
                let app = self.app.clone();
                tokio::spawn(async move {
                    match TargetTcpClient::new(
                        Arc::new(TargetTcpCallbacks::new(app.clone())),
                        id,
                        remote_host_port,
                    )
                    .await
                    {
                        Ok(connection) => {
                            app.tunnel_tcp_connection
                                .new_target_connection_established(&connection)
                                .await;
                        }
                        Err(err) => {
                            app.tunnel_tcp_connection
                                .send_can_not_establish_target_connection_to_tunnel(id, err)
                                .await;
                        }
                    };
                });
            }
            TunnelTcpContract::Connected(_) => {
                // N/A
            }
            TunnelTcpContract::CanNotConnect { id: _, reason: _ } => {
                // N/A
            }
            TunnelTcpContract::DisconnectedFromSideA(id) => {
                // Socket is disconnected on b side

                self.app
                    .tunnel_tcp_connection
                    .disconnect_target_tcp_connection(
                        id,
                        crate::tcp_tunnel::DisconnectReason::DisconnectedFromSideA,
                    )
                    .await;
            }
            TunnelTcpContract::DisconnectedFromSideB(id) => {
                // Socket is disconnected on b side

                self.app
                    .tunnel_tcp_connection
                    .disconnect_target_tcp_connection(
                        id,
                        crate::tcp_tunnel::DisconnectReason::DisconnectedFromSideB,
                    )
                    .await;
            }
            TunnelTcpContract::Payload { id, payload } => {
                // We have payload from a to b;

                self.app
                    .tunnel_tcp_connection
                    .send_payload_to_target(id, payload)
                    .await;
            }
            TunnelTcpContract::Greeting(_) => {
                // N/A
            }
        }
    }
}

#[async_trait::async_trait]
impl SocketEventCallback<TunnelTcpContract, TunnelTcpSerializer> for TunnelTcpEvents {
    async fn handle(
        &self,
        connection_event: ConnectionEvent<TunnelTcpContract, TunnelTcpSerializer>,
    ) {
        match connection_event {
            ConnectionEvent::Connected(connection) => {
                self.app
                    .tunnel_tcp_connection
                    .tunnel_is_connected(connection.clone())
                    .await;
                connection
                    .send(TunnelTcpContract::Greeting(
                        self.app.settings.tunnel_hand_shake_phrase.clone(),
                    ))
                    .await;
            }
            ConnectionEvent::Disconnected(_connection) => {
                self.app
                    .tunnel_tcp_connection
                    .tunnel_is_disconnected()
                    .await;
            }
            ConnectionEvent::Payload {
                connection: _,
                payload,
            } => {
                self.handle_payload(payload).await;
            }
        }
    }
}

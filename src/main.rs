use std::sync::Arc;

use my_tcp_sockets::TcpClient;
use tcp_tunnel::TunnelTcpEvents;
use traffic_forwarder_shared::tcp_tunnel::TunnelTcpSerializer;

mod app;
mod settings_model;
mod target_tcp_client;
mod tcp_tunnel;

#[tokio::main]
async fn main() {
    let settings = crate::settings_model::SettingsModel::load(".traffic-forwarder-b").await;

    let app = app::AppContext::new(settings);
    let app = Arc::new(app);

    let socket = TcpClient::new(
        "TunnelConnection".to_string(),
        app.settings.tunnel_host_port.clone(),
    );

    socket
        .start(
            Arc::new(TunnelTcpSerializer::new),
            Arc::new(TunnelTcpEvents::new(app.clone())),
            my_logger::LOGGER.clone(),
        )
        .await;

    app.app_states.wait_until_shutdown().await;
}

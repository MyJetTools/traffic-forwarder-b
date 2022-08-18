use std::sync::Arc;

use rust_extensions::AppStates;

use crate::{settings_model::SettingsModel, tcp_tunnel::TunnelTcpConnection};

pub struct AppContext {
    pub app_states: Arc<AppStates>,
    pub settings: SettingsModel,
    pub tunnel_tcp_connection: TunnelTcpConnection,
}

impl AppContext {
    pub fn new(settings: SettingsModel) -> Self {
        Self {
            app_states: Arc::new(AppStates::create_initialized()),
            settings,
            tunnel_tcp_connection: TunnelTcpConnection::new(),
        }
    }
}

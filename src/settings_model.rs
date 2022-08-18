use serde_derive::{Deserialize, Serialize};

#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "TunnelHostPort")]
    pub tunnel_host_port: String,
    #[serde(rename = "TunnelHandShakePhrase")]
    pub tunnel_hand_shake_phrase: String,
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "args")]
pub enum IpcCommand {
    ListDevices,
    PairDevice { device_id: Uuid },
    ConfirmPairing { device_id: Uuid, accept: bool },
    SendFile { device_id: Uuid, file_path: String },
    SendText { device_id: Uuid, text: String },
    SendUrl { device_id: Uuid, url: String, title: Option<String> },
    ToggleClipboard { enabled: bool },
    GetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum IpcResponse {
    Ok(serde_json::Value),
    Error { code: String, message: String },
}

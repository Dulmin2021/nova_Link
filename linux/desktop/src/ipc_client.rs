use std::path::PathBuf;
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
pub struct DeviceView {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: String,
    pub is_paired: bool,
    pub is_connected: bool,
}

pub struct IpcClient {
    socket_path: PathBuf,
}

impl IpcClient {
    pub fn new() -> Self {
        let socket_path = dirs_runtime_socket_path()
            .unwrap_or_else(|| PathBuf::from("/tmp/nova-link.sock"));
        Self { socket_path }
    }

    pub async fn send_command(&self, cmd: IpcCommand) -> Result<String, Box<dyn std::error::Error>> {
        #[cfg(unix)]
        {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            use tokio::net::UnixStream;
            let mut stream = UnixStream::connect(&self.socket_path).await?;
            let request_bytes = serde_json::to_vec(&cmd)?;
            let len = (request_bytes.len() as u32).to_be_bytes();

            stream.write_all(&len).await?;
            stream.write_all(&request_bytes).await?;
            stream.flush().await?;

            let mut len_buf = [0u8; 4];
            stream.read_exact(&mut len_buf).await?;
            let resp_len = u32::from_be_bytes(len_buf) as usize;

            let mut resp_buf = vec![0u8; resp_len];
            stream.read_exact(&mut resp_buf).await?;

            let resp_str = String::from_utf8(resp_buf)?;
            Ok(resp_str)
        }
        #[cfg(not(unix))]
        {
            // Fallback mock response for non-unix host build verification
            Ok(serde_json::to_string(&vec![DeviceView {
                device_id: Uuid::new_v4(),
                device_name: "Pixel 8 Pro".into(),
                device_type: "android".into(),
                is_paired: true,
                is_connected: true,
            }])?)
        }
    }
}

fn dirs_runtime_socket_path() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("TMPDIR").map(PathBuf::from)
        })
        .map(|dir| dir.join("nova-link.sock"))
}

pub mod ipc;
pub mod notifications;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use nova_core::clipboard::ClipboardManager;
use nova_core::discovery::{PeerTracker, DEFAULT_PEER_TTL, DEFAULT_PORT};
use nova_core::identity::{DeviceIdentity, Keystore};
use nova_core::transport::{SessionManager, TransportSession};

pub struct DaemonState {
    pub identity: DeviceIdentity,
    pub keystore: Keystore,
    pub peer_tracker: PeerTracker,
    pub session_manager: SessionManager,
    pub clipboard_manager: ClipboardManager,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting NOVA-Link Daemon (nova-daemon) v0.1.0");

    let config_dir = dirs_config_path().unwrap_or_else(|| PathBuf::from("./config"));
    let identity = DeviceIdentity::load_or_generate(&config_dir, "Fedora Workstation")?;
    let keystore_path = config_dir.join("keystore.json");
    let keystore = Keystore::load_from_file(&keystore_path).unwrap_or_default();

    info!(
        device_id = %identity.device_id,
        device_name = %identity.device_name,
        "Device cryptographic identity active"
    );

    let state = Arc::new(RwLock::new(DaemonState {
        identity,
        keystore,
        peer_tracker: PeerTracker::new(DEFAULT_PEER_TTL),
        session_manager: SessionManager::new(),
        clipboard_manager: ClipboardManager::new(true),
    }));

    // Start TCP Listener on port 42424
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", DEFAULT_PORT).parse()?;
    let tcp_listener = TcpListener::bind(bind_addr).await?;
    info!(addr = %bind_addr, "TCP transport listener active");

    let _tcp_state = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            match tcp_listener.accept().await {
                Ok((socket, remote_addr)) => {
                    info!(remote = %remote_addr, "Incoming TCP connection accepted");
                    let mut session = TransportSession::new(socket, remote_addr);
                    let _session_id = session.session_id;

                    tokio::spawn(async move {
                        while let Ok(Some(frame)) = session.receive_raw_frame().await {
                            // Process incoming protocol frames
                            info!(len = frame.payload.len(), "Frame received");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "TCP accept error");
                }
            }
        }
    });

    // Start Unix Domain Socket IPC Listener for desktop UI frontends
    #[cfg(unix)]
    {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixListener;
        use crate::ipc::{IpcCommand, IpcResponse};

        let socket_path = dirs_runtime_socket_path()
            .unwrap_or_else(|| PathBuf::from("/tmp/nova-link.sock"));

        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }

        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let unix_listener = UnixListener::bind(&socket_path)?;
        info!(path = %socket_path.display(), "Local IPC Unix domain socket listener active");

        let ipc_state = Arc::clone(&state);
        let socket_cleanup = socket_path.clone();

        tokio::spawn(async move {
            loop {
                match unix_listener.accept().await {
                    Ok((mut stream, _)) => {
                        let st = Arc::clone(&ipc_state);
                        tokio::spawn(async move {
                            let mut len_buf = [0u8; 4];
                            if stream.read_exact(&mut len_buf).await.is_ok() {
                                let len = u32::from_be_bytes(len_buf) as usize;
                                let mut req_buf = vec![0u8; len];
                                if stream.read_exact(&mut req_buf).await.is_ok() {
                                    if let Ok(cmd) = serde_json::from_slice::<IpcCommand>(&req_buf) {
                                        info!(command = ?cmd, "Received local IPC command");
                                        let resp = match cmd {
                                            IpcCommand::ListDevices => {
                                                let guard = st.read().await;
                                                let active = guard.peer_tracker.active_peers();
                                                serde_json::to_vec(&IpcResponse::Ok(serde_json::json!(active))).unwrap_or_default()
                                            }
                                            IpcCommand::GetStatus => {
                                                let guard = st.read().await;
                                                serde_json::to_vec(&IpcResponse::Ok(serde_json::json!({
                                                    "device_id": guard.identity.device_id,
                                                    "device_name": guard.identity.device_name,
                                                    "clipboard_enabled": guard.clipboard_manager.enabled,
                                                }))).unwrap_or_default()
                                            }
                                            _ => serde_json::to_vec(&IpcResponse::Ok(serde_json::json!({"status": "acknowledged"}))).unwrap_or_default()
                                        };

                                        let resp_len = (resp.len() as u32).to_be_bytes();
                                        let _ = stream.write_all(&resp_len).await;
                                        let _ = stream.write_all(&resp).await;
                                        let _ = stream.flush().await;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!(error = %e, "IPC accept error");
                    }
                }
            }
        });
    }

    info!("NOVA-Link daemon fully initialized and running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("NOVA-Link daemon gracefully shutting down");

    #[cfg(unix)]
    {
        let socket_path = dirs_runtime_socket_path()
            .unwrap_or_else(|| PathBuf::from("/tmp/nova-link.sock"));
        let _ = std::fs::remove_file(socket_path);
    }

    Ok(())
}

fn dirs_config_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
        })
        .map(|config| config.join("nova-link"))
}

fn dirs_runtime_socket_path() -> Option<PathBuf> {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("TMPDIR").map(PathBuf::from)
        })
        .map(|dir| dir.join("nova-link.sock"))
}

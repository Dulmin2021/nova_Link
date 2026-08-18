pub mod ipc;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::RwLock;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

use nova_core::clipboard::ClipboardManager;
use nova_core::discovery::{DiscoveredDevice, PeerTracker, DEFAULT_PEER_TTL, DEFAULT_PORT};
use nova_core::identity::{DeviceIdentity, Keystore};
use nova_core::protocol::{
    DeviceInfoPayload, DeviceType, MessageEnvelope, NovaFrameCodec, RawFrame, PROTOCOL_VERSION,
};
use nova_core::transfer::TransferSession;
use nova_core::transport::{ConnectionState, SessionManager, TransportSession};

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

    let tcp_state = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            match tcp_listener.accept().await {
                Ok((socket, remote_addr)) => {
                    info!(remote = %remote_addr, "Incoming TCP connection accepted");
                    let mut session = TransportSession::new(socket, remote_addr);
                    let session_id = session.session_id;

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

    info!("NOVA-Link daemon fully initialized and running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("NOVA-Link daemon gracefully shutting down");

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

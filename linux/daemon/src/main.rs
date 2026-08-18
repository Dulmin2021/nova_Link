use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use nova_core::identity::DeviceIdentity;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
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

    info!(
        device_id = %identity.device_id,
        device_name = %identity.device_name,
        "Device identity initialized"
    );

    info!("NOVA-Link daemon core initialized and awaiting connections");

    // Daemon lifecycle handler
    tokio::signal::ctrl_c().await?;
    info!("Shutting down NOVA-Link daemon");

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

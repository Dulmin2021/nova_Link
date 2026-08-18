use tracing::info;
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting NOVA-Link Desktop Client (GTK4/Libadwaita frontend)");
    info!("Connecting to local background daemon at IPC socket...");
}

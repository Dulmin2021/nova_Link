use tracing::{info, warn};

pub struct DesktopNotifier;

impl DesktopNotifier {
    pub fn notify_pairing_request(device_name: &str, sas_code: &str) {
        info!(device = %device_name, sas = %sas_code, "Pairing request received");
        #[cfg(unix)]
        {
            // Trigger notify-send on desktop if present
            let summary = format!("NOVA-Link: Pairing with {}", device_name);
            let body = format!("Verification Code: {}\nClick to review.", sas_code);
            let _ = std::process::Command::new("notify-send")
                .args(["-a", "NOVA-Link", "-u", "critical", &summary, &body])
                .spawn();
        }
    }

    pub fn notify_transfer_completed(filename: &str, sender_name: &str) {
        info!(file = %filename, sender = %sender_name, "File transfer completed");
        #[cfg(unix)]
        {
            let summary = "NOVA-Link: File Received";
            let body = format!("Received {} from {}", filename, sender_name);
            let _ = std::process::Command::new("notify-send")
                .args(["-a", "NOVA-Link", "-u", "normal", summary, &body])
                .spawn();
        }
    }

    pub fn notify_url_received(url: &str, sender_name: &str) {
        info!(url = %url, sender = %sender_name, "URL received from peer");
        #[cfg(unix)]
        {
            let summary = "NOVA-Link: Link Received";
            let body = format!("{}\nFrom {}", url, sender_name);
            let _ = std::process::Command::new("notify-send")
                .args(["-a", "NOVA-Link", "-u", "normal", summary, &body])
                .spawn();
        }
    }
}

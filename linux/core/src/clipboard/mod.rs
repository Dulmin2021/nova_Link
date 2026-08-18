use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardManager {
    last_sent_hash: Option<String>,
    last_received_hash: Option<String>,
    pub enabled: bool,
}

impl ClipboardManager {
    pub fn new(enabled: bool) -> Self {
        Self {
            last_sent_hash: None,
            last_received_hash: None,
            enabled,
        }
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn should_broadcast(&mut self, content: &str) -> bool {
        if !self.enabled || content.is_empty() {
            return false;
        }

        let hash = Self::compute_hash(content);
        if let Some(ref last_rx) = self.last_received_hash {
            if last_rx == &hash {
                // Loop prevention: this content originated from peer
                return false;
            }
        }

        if let Some(ref last_tx) = self.last_sent_hash {
            if last_tx == &hash {
                // Duplicate broadcast prevention
                return false;
            }
        }

        self.last_sent_hash = Some(hash);
        true
    }

    pub fn register_received(&mut self, content: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let hash = Self::compute_hash(content);
        if let Some(ref last_tx) = self.last_sent_hash {
            if last_tx == &hash {
                // Reflected our own broadcast
                return false;
            }
        }

        self.last_received_hash = Some(hash);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_loop_prevention() {
        let mut mgr = ClipboardManager::new(true);
        let sample = "https://fedoraproject.org";

        // Initial broadcast is allowed
        assert!(mgr.should_broadcast(sample));
        // Immediate duplicate broadcast is blocked
        assert!(!mgr.should_broadcast(sample));

        // When receiving content, register it
        let incoming = "sudo dnf update";
        assert!(mgr.register_received(incoming));
        // Attempting to echo the received content back out is blocked
        assert!(!mgr.should_broadcast(incoming));
    }
}

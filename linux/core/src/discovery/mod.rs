use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use uuid::Uuid;
use crate::protocol::DeviceType;

pub const SERVICE_TYPE: &str = "_nova-link._tcp.local.";
pub const DEFAULT_PORT: u16 = 42424;
pub const DEFAULT_PEER_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub ip_addresses: Vec<IpAddr>,
    pub port: u16,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
    pub public_key_fingerprint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceAttributes {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub protocol_version: u32,
    pub capabilities: Vec<String>,
    pub fingerprint: String,
}

impl ServiceAttributes {
    pub fn to_txt_records(&self) -> Vec<(String, String)> {
        vec![
            ("id".into(), self.device_id.to_string()),
            ("name".into(), self.device_name.clone()),
            (
                "type".into(),
                match self.device_type {
                    DeviceType::Linux => "linux".into(),
                    DeviceType::Android => "android".into(),
                    DeviceType::Other => "other".into(),
                },
            ),
            ("proto".into(), self.protocol_version.to_string()),
            ("caps".into(), self.capabilities.join(",")),
            ("fp".into(), self.fingerprint.clone()),
        ]
    }

    pub fn from_txt_map(map: &HashMap<String, String>) -> Option<Self> {
        let id_str = map.get("id")?;
        let device_id = Uuid::parse_str(id_str).ok()?;
        let device_name = map.get("name")?.clone();
        let device_type = match map.get("type").map(|s| s.as_str()) {
            Some("android") => DeviceType::Android,
            Some("linux") => DeviceType::Linux,
            _ => DeviceType::Other,
        };
        let protocol_version = map.get("proto").and_then(|p| p.parse().ok()).unwrap_or(1);
        let capabilities = map
            .get("caps")
            .map(|c| c.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let fingerprint = map.get("fp").cloned().unwrap_or_default();

        Some(Self {
            device_id,
            device_name,
            device_type,
            protocol_version,
            capabilities,
            fingerprint,
        })
    }
}

pub struct PeerTracker {
    peers: HashMap<Uuid, (DiscoveredDevice, Instant)>,
    ttl: Duration,
}

impl PeerTracker {
    pub fn new(ttl: Duration) -> Self {
        Self {
            peers: HashMap::new(),
            ttl,
        }
    }

    pub fn update(&mut self, device: DiscoveredDevice) {
        self.peers.insert(device.device_id, (device, Instant::now()));
    }

    pub fn remove(&mut self, device_id: &Uuid) -> Option<DiscoveredDevice> {
        self.peers.remove(device_id).map(|(d, _)| d)
    }

    pub fn prune_expired(&mut self) -> Vec<Uuid> {
        let now = Instant::now();
        let mut expired = Vec::new();
        self.peers.retain(|id, (_, last_seen)| {
            let active = now.duration_since(*last_seen) < self.ttl;
            if !active {
                expired.push(*id);
            }
            active
        });
        expired
    }

    pub fn active_peers(&self) -> Vec<DiscoveredDevice> {
        let now = Instant::now();
        self.peers
            .values()
            .filter_map(|(device, last_seen)| {
                if now.duration_since(*last_seen) < self.ttl {
                    Some(device.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_txt_record_serialization_and_deserialization() {
        let original = ServiceAttributes {
            device_id: Uuid::new_v4(),
            device_name: "Pixel 8 Pro".into(),
            device_type: DeviceType::Android,
            protocol_version: 1,
            capabilities: vec!["file_transfer".into(), "clipboard".into()],
            fingerprint: "abc12345".into(),
        };

        let records = original.to_txt_records();
        let mut map = HashMap::new();
        for (k, v) in records {
            map.insert(k, v);
        }

        let parsed = ServiceAttributes::from_txt_map(&map).expect("Parsing TXT map must succeed");
        assert_eq!(parsed.device_id, original.device_id);
        assert_eq!(parsed.device_name, "Pixel 8 Pro");
        assert_eq!(parsed.device_type, DeviceType::Android);
        assert_eq!(parsed.capabilities.len(), 2);
    }

    #[test]
    fn test_peer_tracker_pruning() {
        let mut tracker = PeerTracker::new(Duration::from_millis(50));
        let dev = DiscoveredDevice {
            device_id: Uuid::new_v4(),
            device_name: "Fedora 40".into(),
            device_type: DeviceType::Linux,
            ip_addresses: vec![],
            port: 42424,
            protocol_version: 1,
            capabilities: vec![],
            public_key_fingerprint: None,
        };

        tracker.update(dev);
        assert_eq!(tracker.active_peers().len(), 1);

        std::thread::sleep(Duration::from_millis(60));
        let expired = tracker.prune_expired();
        assert_eq!(expired.len(), 1);
        assert!(tracker.active_peers().is_empty());
    }
}

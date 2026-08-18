use std::net::IpAddr;
use uuid::Uuid;
use crate::protocol::DeviceType;

pub const SERVICE_TYPE: &str = "_nova-link._tcp.local.";
pub const DEFAULT_PORT: u16 = 42424;

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
}

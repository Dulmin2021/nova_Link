use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageEnvelope<T> {
    pub version: u32,
    pub message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Uuid>,
    pub timestamp: u64,
    pub message_type: String,
    pub payload: T,
}

impl<T> MessageEnvelope<T> {
    pub fn new(message_type: impl Into<String>, payload: T) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            version: PROTOCOL_VERSION,
            message_id: Uuid::new_v4(),
            reply_to: None,
            timestamp,
            message_type: message_type.into(),
            payload,
        }
    }

    pub fn new_reply(message_type: impl Into<String>, reply_to: Uuid, payload: T) -> Self {
        let mut env = Self::new(message_type, payload);
        env.reply_to = Some(reply_to);
        env
    }
}

// -------------------------------------------------------------------------
// Device Info & Capabilities
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInfoPayload {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub protocol_version: u32,
    #[serde(default)]
    pub os_version: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Linux,
    Android,
    Other,
}

// -------------------------------------------------------------------------
// Pairing Payloads
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingRequestPayload {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub identity_pubkey: String,   // Hex-encoded Ed25519 public key
    pub ephemeral_pubkey: String,  // Hex-encoded X25519 public key
    pub nonce: String,             // Hex-encoded 32-byte nonce
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingResponsePayload {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub identity_pubkey: String,   // Hex-encoded Ed25519 public key
    pub ephemeral_pubkey: String,  // Hex-encoded X25519 public key
    pub nonce: String,             // Hex-encoded 32-byte nonce
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingConfirmPayload {
    pub accepted: bool,
    pub signature: String,         // Hex-encoded signature of confirmation token
}

// -------------------------------------------------------------------------
// Feature Payloads: Clipboard, URL, Text
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardSyncPayload {
    pub content_type: String,
    pub content: String,
    pub checksum: String,          // SHA-256 hash of content
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UrlSharePayload {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextSharePayload {
    pub text: String,
}

// -------------------------------------------------------------------------
// File Transfer Payloads
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferInitPayload {
    pub transfer_id: Uuid,
    pub filename: String,
    pub file_size: u64,
    pub sha256_hash: String,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferAcceptPayload {
    pub transfer_id: Uuid,
    pub accepted: bool,
    pub resume_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferChunkHeader {
    pub transfer_id: Uuid,
    pub chunk_index: u64,
    pub offset: u64,
    pub chunk_length: u32,
    pub is_last_chunk: bool,
    pub chunk_checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferProgressPayload {
    pub transfer_id: Uuid,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub bytes_per_second: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferCompletePayload {
    pub transfer_id: Uuid,
    pub success: bool,
    pub checksum_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferCancelPayload {
    pub transfer_id: Uuid,
    pub reason: String,
}

// -------------------------------------------------------------------------
// Diagnostic & Error Payloads
// -------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmptyPayload {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_envelope_serialization() {
        let payload = DeviceInfoPayload {
            device_id: Uuid::new_v4(),
            device_name: "Fedora Laptop".into(),
            device_type: DeviceType::Linux,
            protocol_version: 1,
            os_version: Some("Fedora 40".into()),
            capabilities: vec!["file_transfer".into(), "clipboard".into()],
        };

        let env = MessageEnvelope::new("device_info", payload);
        let json = serde_json::to_string(&env).expect("serialization should work");
        assert!(json.contains("device_info"));

        let deserialized: MessageEnvelope<DeviceInfoPayload> =
            serde_json::from_str(&json).expect("deserialization should work");
        assert_eq!(deserialized.payload.device_name, "Fedora Laptop");
        assert_eq!(deserialized.payload.device_type, DeviceType::Linux);
    }
}

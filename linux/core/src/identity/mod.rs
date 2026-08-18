use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: Uuid,
    pub device_name: String,
    #[serde(skip)]
    pub signing_key: Option<SigningKey>,
    pub public_key_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedPeer {
    pub device_id: Uuid,
    pub device_name: String,
    pub public_key_hex: String,
    pub paired_at: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Keystore {
    pub trusted_peers: HashMap<Uuid, TrustedPeer>,
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            trusted_peers: HashMap::new(),
        }
    }

    pub fn is_trusted(&self, device_id: &Uuid, public_key_hex: &str) -> bool {
        if let Some(peer) = self.trusted_peers.get(device_id) {
            peer.public_key_hex.eq_ignore_ascii_case(public_key_hex)
        } else {
            false
        }
    }

    pub fn get_peer(&self, device_id: &Uuid) -> Option<&TrustedPeer> {
        self.trusted_peers.get(device_id)
    }

    pub fn add_peer(&mut self, peer: TrustedPeer) {
        self.trusted_peers.insert(peer.device_id, peer);
    }

    pub fn remove_peer(&mut self, device_id: &Uuid) -> Option<TrustedPeer> {
        self.trusted_peers.remove(device_id)
    }

    pub fn list_peers(&self) -> Vec<TrustedPeer> {
        self.trusted_peers.values().cloned().collect()
    }

    pub fn save_to_file(&self, path: &Path) -> NovaResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> NovaResult<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let keystore: Keystore = serde_json::from_str(&contents)?;
        Ok(keystore)
    }
}

impl DeviceIdentity {
    pub fn generate_ephemeral(device_name: impl Into<String>) -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let public_key_hex = hex_encode(verifying_key.to_bytes());

        Self {
            device_id: Uuid::new_v4(),
            device_name: device_name.into(),
            signing_key: Some(signing_key),
            public_key_hex,
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> NovaResult<String> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| NovaError::Crypto("No private signing key loaded".into()))?;
        let signature = signing_key.sign(message);
        Ok(hex_encode(signature.to_bytes()))
    }

    pub fn verify_signature(
        public_key_hex: &str,
        message: &[u8],
        signature_hex: &str,
    ) -> NovaResult<bool> {
        let pk_bytes = hex_decode(public_key_hex)
            .map_err(|e| NovaError::Crypto(format!("Invalid public key hex: {}", e)))?;
        if pk_bytes.len() != 32 {
            return Err(NovaError::Crypto("Public key must be 32 bytes".into()));
        }

        let mut pk_arr = [0u8; 32];
        pk_arr.copy_from_slice(&pk_bytes);
        let verifying_key = VerifyingKey::from_bytes(&pk_arr)
            .map_err(|e| NovaError::Crypto(format!("Invalid VerifyingKey: {}", e)))?;

        let sig_bytes = hex_decode(signature_hex)
            .map_err(|e| NovaError::Crypto(format!("Invalid signature hex: {}", e)))?;
        if sig_bytes.len() != 64 {
            return Err(NovaError::Crypto("Signature must be 64 bytes".into()));
        }

        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        Ok(verifying_key.verify(message, &signature).is_ok())
    }

    pub fn load_or_generate(config_dir: &Path, default_name: &str) -> NovaResult<Self> {
        fs::create_dir_all(config_dir)?;
        let key_path = config_dir.join("identity.key");
        let info_path = config_dir.join("device.json");

        if key_path.exists() && info_path.exists() {
            let mut key_bytes = Vec::new();
            File::open(&key_path)?.read_to_end(&mut key_bytes)?;
            if key_bytes.len() != 32 {
                return Err(NovaError::Crypto("Corrupted private key file".into()));
            }

            let mut arr = [0u8; 32];
            arr.copy_from_slice(&key_bytes);
            let signing_key = SigningKey::from_bytes(&arr);
            let verifying_key = signing_key.verifying_key();
            let public_key_hex = hex_encode(verifying_key.to_bytes());

            let mut info_str = String::new();
            File::open(&info_path)?.read_to_string(&mut info_str)?;
            let mut identity: DeviceIdentity = serde_json::from_str(&info_str)?;
            identity.signing_key = Some(signing_key);
            identity.public_key_hex = public_key_hex;

            Ok(identity)
        } else {
            let identity = Self::generate_ephemeral(default_name);
            if let Some(ref sk) = identity.signing_key {
                let mut f = File::create(&key_path)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = f.metadata()?.permissions();
                    perms.set_mode(0o600);
                    fs::set_permissions(&key_path, perms)?;
                }
                f.write_all(&sk.to_bytes())?;
            }

            let info_json = serde_json::to_string_pretty(&identity)?;
            File::create(&info_path)?.write_all(info_json.as_bytes())?;

            Ok(identity)
        }
    }
}

pub fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

pub fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("Odd hex string length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at index {}: {}", i, e))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_signing_and_verification() {
        let identity = DeviceIdentity::generate_ephemeral("Test Laptop");
        let payload = b"NOVA-LINK-CONFIRM-TOKEN-123";

        let signature = identity.sign_message(payload).expect("Signing must succeed");
        assert_eq!(signature.len(), 128); // 64 bytes in hex

        let valid = DeviceIdentity::verify_signature(&identity.public_key_hex, payload, &signature)
            .expect("Verification must succeed");
        assert!(valid, "Signature verification should pass");

        let invalid = DeviceIdentity::verify_signature(
            &identity.public_key_hex,
            b"TAMPERED-PAYLOAD",
            &signature,
        )
        .expect("Verification must succeed");
        assert!(!invalid, "Tampered payload verification must fail");
    }
}

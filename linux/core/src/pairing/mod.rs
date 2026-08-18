use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use crate::error::{NovaError, NovaResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingState {
    Idle,
    AwaitingResponse,
    AwaitingUserApproval { sas_code: String },
    AwaitingPeerConfirmation,
    Paired,
    Failed(String),
}

pub struct PairingSession {
    ephemeral_secret: Option<EphemeralSecret>,
    pub local_ephemeral_pubkey: [u8; 32],
    pub local_nonce: [u8; 32],
    pub peer_ephemeral_pubkey: Option<[u8; 32]>,
    pub peer_nonce: Option<[u8; 32]>,
    pub shared_secret: Option<[u8; 32]>,
    pub sas_code: Option<String>,
    pub state: PairingState,
}

impl PairingSession {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let ephemeral_secret = EphemeralSecret::random_from_rng(&mut csprng);
        let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

        let mut local_nonce = [0u8; 32];
        csprng.fill_bytes(&mut local_nonce);

        Self {
            ephemeral_secret: Some(ephemeral_secret),
            local_ephemeral_pubkey: *ephemeral_public.as_bytes(),
            local_nonce,
            peer_ephemeral_pubkey: None,
            peer_nonce: None,
            shared_secret: None,
            sas_code: None,
            state: PairingState::Idle,
        }
    }

    pub fn compute_sas(
        &mut self,
        local_identity_pk: &[u8; 32],
        peer_identity_pk: &[u8; 32],
        peer_ephemeral_pk: [u8; 32],
        peer_nonce: [u8; 32],
    ) -> NovaResult<String> {
        let ephemeral_secret = self
            .ephemeral_secret
            .take()
            .ok_or_else(|| NovaError::Pairing("Ephemeral secret already consumed".into()))?;

        let peer_pub = X25519PublicKey::from(peer_ephemeral_pk);
        let shared_secret = ephemeral_secret.diffie_hellman(&peer_pub);
        let shared_secret_bytes = *shared_secret.as_bytes();

        self.peer_ephemeral_pubkey = Some(peer_ephemeral_pk);
        self.peer_nonce = Some(peer_nonce);
        self.shared_secret = Some(shared_secret_bytes);

        // Derive transcript hash
        let mut hasher = Sha256::new();
        hasher.update(local_identity_pk);
        hasher.update(peer_identity_pk);
        hasher.update(&self.local_ephemeral_pubkey);
        hasher.update(&peer_ephemeral_pk);
        hasher.update(&self.local_nonce);
        hasher.update(&peer_nonce);
        hasher.update(&shared_secret_bytes);
        let transcript_hash = hasher.finalize();

        // Derive SAS via HKDF
        let hk = Hkdf::<Sha256>::new(None, &transcript_hash);
        let mut okm = [0u8; 4];
        hk.expand(b"NOVA-LINK-SAS-V1", &mut okm)
            .map_err(|_| NovaError::Crypto("HKDF expansion failed".into()))?;

        let raw_num = u32::from_be_bytes(okm) % 1_000_000;
        let formatted_sas = format!("{:03} {:03}", raw_num / 1000, raw_num % 1000);

        self.sas_code = Some(formatted_sas.clone());
        self.state = PairingState::AwaitingUserApproval {
            sas_code: formatted_sas.clone(),
        };

        Ok(formatted_sas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutual_sas_derivation() {
        let mut session_a = PairingSession::new();
        let mut session_b = PairingSession::new();

        let id_a = [1u8; 32];
        let id_b = [2u8; 32];

        // Session A computes SAS with B's public params
        let sas_a = session_a
            .compute_sas(
                &id_a,
                &id_b,
                session_b.local_ephemeral_pubkey,
                session_b.local_nonce,
            )
            .expect("Session A SAS derivation should succeed");

        // Session B computes SAS with A's public params
        let sas_b = session_b
            .compute_sas(
                &id_a,
                &id_b,
                session_a.local_ephemeral_pubkey,
                session_a.local_nonce,
            )
            .expect("Session B SAS derivation should succeed");

        assert_eq!(sas_a, sas_b, "Both sides must derive identical SAS verification codes");
        assert_eq!(sas_a.len(), 7); // "XXX XXX"
    }
}

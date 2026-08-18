use hkdf::Hkdf;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use crate::error::{NovaError, NovaResult};
use crate::identity::DeviceIdentity;

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
    pub transcript_hash: Option<[u8; 32]>,
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
            transcript_hash: None,
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
        let transcript_hash: [u8; 32] = hasher.finalize().into();
        self.transcript_hash = Some(transcript_hash);

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

    pub fn generate_confirmation_token(&self, role_tag: &[u8]) -> NovaResult<[u8; 32]> {
        let transcript_hash = self
            .transcript_hash
            .as_ref()
            .ok_or_else(|| NovaError::Pairing("Transcript hash not yet computed".into()))?;

        let hk = Hkdf::<Sha256>::new(None, transcript_hash);
        let mut token = [0u8; 32];
        hk.expand(role_tag, &mut token)
            .map_err(|_| NovaError::Crypto("HKDF confirmation expansion failed".into()))?;
        Ok(token)
    }

    pub fn verify_peer_confirmation(
        &self,
        peer_identity_pk_hex: &str,
        peer_signature_hex: &str,
        peer_role_tag: &[u8],
    ) -> NovaResult<bool> {
        let token = self.generate_confirmation_token(peer_role_tag)?;
        DeviceIdentity::verify_signature(peer_identity_pk_hex, &token, peer_signature_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::hex_decode;

    #[test]
    fn test_full_pairing_handshake_flow() {
        let id_a = DeviceIdentity::generate_ephemeral("Fedora Host");
        let id_b = DeviceIdentity::generate_ephemeral("Android Phone");

        let mut session_a = PairingSession::new();
        let mut session_b = PairingSession::new();

        let mut id_a_pk = [0u8; 32];
        let mut id_b_pk = [0u8; 32];
        id_a_pk.copy_from_slice(&hex_decode(&id_a.public_key_hex).unwrap());
        id_b_pk.copy_from_slice(&hex_decode(&id_b.public_key_hex).unwrap());

        // Step 1: Both calculate SAS
        let sas_a = session_a
            .compute_sas(
                &id_a_pk,
                &id_b_pk,
                session_b.local_ephemeral_pubkey,
                session_b.local_nonce,
            )
            .expect("Session A SAS computation");

        let sas_b = session_b
            .compute_sas(
                &id_a_pk,
                &id_b_pk,
                session_a.local_ephemeral_pubkey,
                session_a.local_nonce,
            )
            .expect("Session B SAS computation");

        assert_eq!(sas_a, sas_b);

        // Step 2: Confirmation tokens
        let confirm_a_token = session_a.generate_confirmation_token(b"CONFIRM-A").unwrap();
        let sig_a = id_a.sign_message(&confirm_a_token).unwrap();

        let confirm_b_token = session_b.generate_confirmation_token(b"CONFIRM-B").unwrap();
        let sig_b = id_b.sign_message(&confirm_b_token).unwrap();

        // Step 3: Mutual verification
        let verified_on_b = session_b
            .verify_peer_confirmation(&id_a.public_key_hex, &sig_a, b"CONFIRM-A")
            .unwrap();
        let verified_on_a = session_a
            .verify_peer_confirmation(&id_b.public_key_hex, &sig_b, b"CONFIRM-B")
            .unwrap();

        assert!(verified_on_b, "B must verify A's confirmation signature");
        assert!(verified_on_a, "A must verify B's confirmation signature");
    }
}

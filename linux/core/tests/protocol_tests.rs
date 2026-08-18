use bytes::BytesMut;
use nova_core::identity::DeviceIdentity;
use nova_core::pairing::PairingSession;
use nova_core::protocol::{MessageEnvelope, NovaFrameCodec, RawFrame, UrlSharePayload};
use tokio_util::codec::{Decoder, Encoder};

#[test]
fn test_end_to_end_envelope_framing() {
    let mut codec = NovaFrameCodec::default();
    let mut buffer = BytesMut::new();

    // 1. Construct typed payload
    let payload = UrlSharePayload {
        url: "https://fedoraproject.org".into(),
        title: Some("Fedora Project".into()),
    };

    let envelope = MessageEnvelope::new("url_share", payload);
    let serialized_json = serde_json::to_vec(&envelope).expect("JSON serialization failed");

    // 2. Encode to binary frame
    let frame = RawFrame::new(serialized_json);
    codec.encode(frame, &mut buffer).expect("Encoding failed");

    // 3. Decode frame
    let decoded_frame = codec
        .decode(&mut buffer)
        .expect("Decoding failed")
        .expect("Frame should be complete");

    // 4. Deserialize envelope
    let decoded_envelope: MessageEnvelope<UrlSharePayload> =
        serde_json::from_slice(&decoded_frame.payload).expect("Deserialization failed");

    assert_eq!(decoded_envelope.message_type, "url_share");
    assert_eq!(
        decoded_envelope.payload.url,
        "https://fedoraproject.org"
    );
}

#[test]
fn test_pairing_handshake_simulation() {
    let device_a = DeviceIdentity::generate_ephemeral("Fedora Laptop");
    let device_b = DeviceIdentity::generate_ephemeral("Pixel 8");

    let mut session_a = PairingSession::new();
    let mut session_b = PairingSession::new();

    // Mock 32-byte identity pubkeys
    let mut id_a = [0u8; 32];
    let mut id_b = [0u8; 32];
    id_a.copy_from_slice(&device_a.signing_key.as_ref().unwrap().verifying_key().to_bytes());
    id_b.copy_from_slice(&device_b.signing_key.as_ref().unwrap().verifying_key().to_bytes());

    let sas_a = session_a
        .compute_sas(
            &id_a,
            &id_b,
            session_b.local_ephemeral_pubkey,
            session_b.local_nonce,
        )
        .expect("SAS computation on A failed");

    let sas_b = session_b
        .compute_sas(
            &id_b,
            &id_a,
            session_a.local_ephemeral_pubkey,
            session_a.local_nonce,
        )
        .expect("SAS computation on B failed");

    assert_eq!(sas_a, sas_b, "SAS verification codes must match perfectly");
    assert_eq!(sas_a.len(), 7, "SAS format must be 'XXX XXX'");
}

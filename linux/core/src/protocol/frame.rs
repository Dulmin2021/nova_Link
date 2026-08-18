use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use crate::error::{NovaError, NovaResult};

pub const MAGIC_BYTES: [u8; 2] = [0x4E, 0x4C]; // "NL"
pub const HEADER_LEN: usize = 6; // 2 bytes magic + 4 bytes length
pub const DEFAULT_MAX_FRAME_SIZE: usize = 16 * 1024 * 1024; // 16 MB max frame size

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFrame {
    pub payload: Vec<u8>,
}

impl RawFrame {
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self { payload: bytes.to_vec() }
    }
}

pub struct NovaFrameCodec {
    max_frame_size: usize,
}

impl Default for NovaFrameCodec {
    fn default() -> Self {
        Self {
            max_frame_size: DEFAULT_MAX_FRAME_SIZE,
        }
    }
}

impl NovaFrameCodec {
    pub fn new(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }
}

impl Decoder for NovaFrameCodec {
    type Item = RawFrame;
    type Error = NovaError;

    fn decode(&mut self, src: &mut BytesMut) -> NovaResult<Option<Self::Item>> {
        if src.len() < HEADER_LEN {
            return Ok(None);
        }

        // Check magic bytes
        if src[0..2] != MAGIC_BYTES {
            return Err(NovaError::InvalidFrame(format!(
                "Invalid magic header: expected [0x4E, 0x4C], got [0x{:02X}, 0x{:02X}]",
                src[0], src[1]
            )));
        }

        // Read payload length (big-endian uint32)
        let payload_len = u32::from_be_bytes([src[2], src[3], src[4], src[5]]) as usize;

        if payload_len > self.max_frame_size {
            return Err(NovaError::FrameTooLarge(payload_len, self.max_frame_size));
        }

        let total_len = HEADER_LEN + payload_len;
        if src.len() < total_len {
            // Wait for full frame bytes to arrive
            src.reserve(total_len - src.len());
            return Ok(None);
        }

        // Advance past header
        src.advance(HEADER_LEN);

        // Split payload bytes
        let payload = src.split_to(payload_len).to_vec();

        Ok(Some(RawFrame { payload }))
    }
}

impl Encoder<RawFrame> for NovaFrameCodec {
    type Error = NovaError;

    fn encode(&mut self, item: RawFrame, dst: &mut BytesMut) -> NovaResult<()> {
        let payload_len = item.payload.len();
        if payload_len > self.max_frame_size {
            return Err(NovaError::FrameTooLarge(payload_len, self.max_frame_size));
        }

        dst.reserve(HEADER_LEN + payload_len);
        dst.put_slice(&MAGIC_BYTES);
        dst.put_u32(payload_len as u32);
        dst.put_slice(&item.payload);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let mut codec = NovaFrameCodec::default();
        let mut buf = BytesMut::new();

        let original_data = b"{\"version\":1,\"message_type\":\"ping\"}".to_vec();
        let frame = RawFrame::new(original_data.clone());

        codec.encode(frame, &mut buf).expect("encoding should succeed");
        assert_eq!(buf.len(), HEADER_LEN + original_data.len());
        assert_eq!(&buf[0..2], &MAGIC_BYTES);

        let decoded = codec.decode(&mut buf).expect("decoding should succeed");
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap().payload, original_data);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_partial_frame_decoding() {
        let mut codec = NovaFrameCodec::default();
        let mut buf = BytesMut::new();

        let original_data = b"Hello, NOVA-Link!".to_vec();
        let frame = RawFrame::new(original_data.clone());

        codec.encode(frame, &mut buf).expect("encoding should succeed");

        // Split buffer into partial pieces
        let mut partial = buf.split_to(4); // only part of header
        assert!(codec.decode(&mut partial).unwrap().is_none());

        partial.unsplit(buf); // restore remainder
        let decoded = codec.decode(&mut partial).expect("decoding should succeed");
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap().payload, original_data);
    }

    #[test]
    fn test_invalid_magic_fails() {
        let mut codec = NovaFrameCodec::default();
        let mut buf = BytesMut::from(&[0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x41][..]);

        let result = codec.decode(&mut buf);
        assert!(result.is_err());
    }
}

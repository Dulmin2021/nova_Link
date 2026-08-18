pub mod frame;
pub mod messages;

pub use frame::{NovaFrameCodec, RawFrame, DEFAULT_MAX_FRAME_SIZE, HEADER_LEN, MAGIC_BYTES};
pub use messages::{
    ClipboardSyncPayload, DeviceInfoPayload, DeviceType, EmptyPayload, ErrorPayload,
    MessageEnvelope, PairingConfirmPayload, PairingRequestPayload, PairingResponsePayload,
    TextSharePayload, TransferAcceptPayload, TransferCancelPayload, TransferChunkHeader,
    TransferCompletePayload, TransferInitPayload, TransferProgressPayload, UrlSharePayload,
    PROTOCOL_VERSION,
};

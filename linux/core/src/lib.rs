//! NOVA-Link Core Library (`nova-core`)
//!
//! Provides the core networking, cryptographic identity, framing,
//! pairing, and protocol engines for NOVA-Link on Linux.

pub mod clipboard;
pub mod discovery;
pub mod error;
pub mod identity;
pub mod pairing;
pub mod protocol;
pub mod transfer;
pub mod transport;

pub use error::{NovaError, NovaResult};
pub use identity::{DeviceIdentity, Keystore, TrustedPeer};
pub use pairing::{PairingSession, PairingState};
pub use protocol::*;
pub use transfer::TransferSession;

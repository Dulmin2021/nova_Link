use thiserror::Error;

#[derive(Error, Debug)]
pub enum NovaError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid protocol frame: {0}")]
    InvalidFrame(String),

    #[error("Frame too large: {0} bytes (max: {1})")]
    FrameTooLarge(usize, usize),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Pairing error: {0}")]
    Pairing(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type NovaResult<T> = Result<T, NovaError>;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};
use crate::identity::hex_encode;

pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024; // 64 KiB

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    Outgoing,
    Incoming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug)]
pub struct TransferSession {
    pub transfer_id: Uuid,
    pub direction: TransferDirection,
    pub filename: String,
    pub total_bytes: u64,
    pub bytes_transferred: u64,
    pub expected_sha256: String,
    pub local_path: PathBuf,
    pub status: TransferStatus,
    hasher: Sha256,
    file_handle: Option<File>,
}

impl TransferSession {
    pub fn new_incoming(
        transfer_id: Uuid,
        filename: String,
        total_bytes: u64,
        expected_sha256: String,
        destination_dir: &Path,
    ) -> NovaResult<Self> {
        let sanitized = sanitize_filename(&filename);
        let target_path = resolve_unique_path(destination_dir, &sanitized);
        let part_path = target_path.with_extension("nova_part");

        if let Some(parent) = part_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(&part_path)?;

        Ok(Self {
            transfer_id,
            direction: TransferDirection::Incoming,
            filename: sanitized,
            total_bytes,
            bytes_transferred: 0,
            expected_sha256,
            local_path: target_path,
            status: TransferStatus::InProgress,
            hasher: Sha256::new(),
            file_handle: Some(file),
        })
    }

    pub fn new_outgoing(
        transfer_id: Uuid,
        source_path: &Path,
    ) -> NovaResult<(Self, String)> {
        let mut file = File::open(source_path)?;
        let metadata = file.metadata()?;
        let total_bytes = metadata.len();

        let filename = source_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();

        // Calculate SHA-256
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let sha256_hex = hex_encode(hasher.finalize());

        // Rewind file
        file.seek(SeekFrom::Start(0))?;

        let session = Self {
            transfer_id,
            direction: TransferDirection::Outgoing,
            filename,
            total_bytes,
            bytes_transferred: 0,
            expected_sha256: sha256_hex.clone(),
            local_path: source_path.to_path_buf(),
            status: TransferStatus::Pending,
            hasher: Sha256::new(),
            file_handle: Some(file),
        };

        Ok((session, sha256_hex))
    }

    pub fn write_incoming_chunk(&mut self, chunk_bytes: &[u8]) -> NovaResult<()> {
        if self.status != TransferStatus::InProgress {
            return Err(NovaError::Transfer("Transfer is not in progress".into()));
        }

        let file = self
            .file_handle
            .as_mut()
            .ok_or_else(|| NovaError::Transfer("Missing active file handle".into()))?;

        file.write_all(chunk_bytes)?;
        self.hasher.update(chunk_bytes);
        self.bytes_transferred += chunk_bytes.len() as u64;

        Ok(())
    }

    pub fn finalize_incoming(&mut self) -> NovaResult<bool> {
        if let Some(mut file) = self.file_handle.take() {
            file.flush()?;
        }

        let calculated_hash = hex_encode(self.hasher.clone().finalize());
        let part_path = self.local_path.with_extension("nova_part");

        if calculated_hash.eq_ignore_ascii_case(&self.expected_sha256) {
            std::fs::rename(&part_path, &self.local_path)?;
            self.status = TransferStatus::Completed;
            Ok(true)
        } else {
            let _ = std::fs::remove_file(&part_path);
            self.status = TransferStatus::Failed;
            Err(NovaError::Transfer(format!(
                "Checksum mismatch: expected {}, got {}",
                self.expected_sha256, calculated_hash
            )))
        }
    }

    pub fn read_next_outgoing_chunk(&mut self, chunk_size: usize) -> NovaResult<Option<(Vec<u8>, bool)>> {
        let file = self
            .file_handle
            .as_mut()
            .ok_or_else(|| NovaError::Transfer("Missing active file handle".into()))?;

        let mut buffer = vec![0u8; chunk_size];
        let n = file.read(&mut buffer)?;
        if n == 0 {
            self.status = TransferStatus::Completed;
            return Ok(None);
        }

        buffer.truncate(n);
        self.bytes_transferred += n as u64;
        let is_last = self.bytes_transferred >= self.total_bytes;
        if is_last {
            self.status = TransferStatus::Completed;
        }

        Ok(Some((buffer, is_last)))
    }

    pub fn cancel(&mut self) -> NovaResult<()> {
        self.status = TransferStatus::Cancelled;
        self.file_handle = None;
        if self.direction == TransferDirection::Incoming {
            let part_path = self.local_path.with_extension("nova_part");
            let _ = std::fs::remove_file(part_path);
        }
        Ok(())
    }

    pub fn progress_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            1.0
        } else {
            (self.bytes_transferred as f64) / (self.total_bytes as f64)
        }
    }
}

pub fn sanitize_filename(name: &str) -> String {
    let clean = name
        .replace('/', "_")
        .replace('\\', "_")
        .replace('\0', "")
        .replace("..", "_");
    if clean.trim().is_empty() {
        "unnamed_file".to_string()
    } else {
        clean
    }
}

pub fn resolve_unique_path(dir: &Path, filename: &str) -> PathBuf {
    let mut candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_name = format!("{} ({}){}", stem, counter, extension);
        candidate = dir.join(new_name);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_transfer_chunk_pipeline_and_checksum_verification() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("source.dat");
        let dest_dir = temp_dir.path().join("downloads");
        std::fs::create_dir_all(&dest_dir).unwrap();

        // Create sample 256 KiB file
        let sample_data = vec![0xAB; 256 * 1024];
        {
            let mut f = File::create(&src_path).unwrap();
            f.write_all(&sample_data).unwrap();
        }

        let transfer_id = Uuid::new_v4();

        // Sender side
        let (mut sender_session, sha256_hex) =
            TransferSession::new_outgoing(transfer_id, &src_path).unwrap();
        assert_eq!(sender_session.total_bytes, 256 * 1024);

        // Receiver side
        let mut receiver_session = TransferSession::new_incoming(
            transfer_id,
            "source.dat".into(),
            sender_session.total_bytes,
            sha256_hex,
            &dest_dir,
        )
        .unwrap();

        // Stream chunks in 64 KiB pieces
        while let Some((chunk, _is_last)) = sender_session.read_next_outgoing_chunk(64 * 1024).unwrap() {
            receiver_session.write_incoming_chunk(&chunk).unwrap();
        }

        assert_eq!(receiver_session.bytes_transferred, 256 * 1024);
        let finalized = receiver_session.finalize_incoming().unwrap();
        assert!(finalized);
        assert_eq!(receiver_session.status, TransferStatus::Completed);

        // Verify content on disk
        let mut received_bytes = Vec::new();
        File::open(&receiver_session.local_path)
            .unwrap()
            .read_to_end(&mut received_bytes)
            .unwrap();
        assert_eq!(received_bytes, sample_data);
    }
}

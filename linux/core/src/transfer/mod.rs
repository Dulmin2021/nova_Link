use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use crate::error::{NovaError, NovaResult};

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

#[derive(Debug, Clone)]
pub struct TransferSession {
    pub transfer_id: Uuid,
    pub direction: TransferDirection,
    pub filename: String,
    pub total_bytes: u64,
    pub bytes_transferred: u64,
    pub expected_sha256: String,
    pub local_path: PathBuf,
    pub status: TransferStatus,
}

impl TransferSession {
    pub fn new_incoming(
        transfer_id: Uuid,
        filename: String,
        total_bytes: u64,
        expected_sha256: String,
        destination_dir: &Path,
    ) -> Self {
        let sanitized_filename = sanitize_filename(&filename);
        let target_path = resolve_unique_path(destination_dir, &sanitized_filename);

        Self {
            transfer_id,
            direction: TransferDirection::Incoming,
            filename: sanitized_filename,
            total_bytes,
            bytes_transferred: 0,
            expected_sha256,
            local_path: target_path,
            status: TransferStatus::Pending,
        }
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

    #[test]
    fn test_sanitize_filename_traversal() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "____etc_passwd");
        assert_eq!(sanitize_filename("valid_image.png"), "valid_image.png");
    }

    #[test]
    fn test_resolve_unique_path_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let p = resolve_unique_path(temp_dir.path(), "test.txt");
        assert_eq!(p, temp_dir.path().join("test.txt"));
    }
}

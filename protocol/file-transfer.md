# NOVA-Link Streaming File Transfer Protocol Specification

## 1. Overview

NOVA-Link file transfers operate as non-blocking streaming data pipelines capable of transferring multi-gigabyte files (e.g. 10GB+ ISO images, video recordings) with minimal RAM overhead.

---

## 2. Transfer Lifecycle

```
Sender                                                         Receiver
  |                                                                |
  | 1. transfer_init                                               |
  |    { transfer_id, filename, file_size, sha256_hash, mime }    |
  |--------------------------------------------------------------->|
  |                                                                |
  |                                  [Prompt User / Check Space /  |
  |                                   Resolve Filename Collisions] |
  |                                                                |
  | 2. transfer_accept / transfer_reject                           |
  |    { transfer_id, accepted: true, resume_offset: 0 }           |
  |<---------------------------------------------------------------|
  |                                                                |
  | 3. Stream binary chunks (0x4E4C framing)                       |
  |    [Chunk 0: 64 KiB .. 1 MiB]                                  |
  |--------------------------------------------------------------->|
  |    [Chunk 1: 64 KiB .. 1 MiB]                                  |
  |--------------------------------------------------------------->|
  |    ...                                                         |
  |                                                                |
  | 4. transfer_progress (Periodic ACK / Speed & Offset Sync)      |
  |<---------------------------------------------------------------|
  |                                                                |
  | 5. transfer_complete                                           |
  |    { transfer_id, status: "success", checksum_verified: true } |
  |--------------------------------------------------------------->|
```

---

## 3. Chunk Header Structure

File stream chunks are sent using the protocol framing format where each chunk payload begins with a standard binary or JSON-prefixed chunk header:

```json
{
  "version": 1,
  "message_id": "uuid-v4",
  "reply_to": null,
  "timestamp": 1723974630,
  "message_type": "transfer_chunk",
  "payload": {
    "transfer_id": "transfer-uuid-v4",
    "chunk_index": 42,
    "offset": 2752512,
    "chunk_length": 65536,
    "is_last_chunk": false,
    "chunk_checksum": "blake3_or_sha256_hex"
  }
}
```

Followed immediately by `chunk_length` raw bytes in the data frame.

---

## 4. Error Handling & Flow Control

1. **Cancellation**: Either party can transmit `transfer_cancel` at any point with reason (`USER_CANCELLED`, `DISK_FULL`, `TIMEOUT`). The sender terminates streaming immediately, and the receiver deletes the partial temporary file (`.nova_part`).
2. **Duplicate Filename Policy**: If a file `example.jpg` exists in the target directory:
   * Automatic collision avoidance generates `example (1).jpg`.
   * UI displays option: Replace, Keep Both, or Cancel.
3. **Integrity Verification**: The receiver streams bytes into a local SHA-256 / BLAKE3 hasher. Upon receiving `transfer_complete`, it validates the computed hash against the initial `transfer_init` hash before finalizing the file.

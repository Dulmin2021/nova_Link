# NOVA-Link Protocol Specification (v1.0)

## 1. Overview

The NOVA-Link Protocol specifies how Android and Linux nodes discover, pair, authenticate, and communicate over a Local Area Network (LAN).

The protocol is designed to be:
* **Asynchronous & Non-Blocking**: Built on full-duplex framing.
* **Extensible**: Structured with capability negotiation and typed messages.
* **Resilient**: Immune to packet truncation, deserialization bombs, and framing exploits.
* **Secure**: Zero-trust authenticated key exchange with forward secrecy.

---

## 2. Protocol Documents

| Document | Description |
| :--- | :--- |
| [**discovery.md**](./discovery.md) | mDNS / DNS-SD service announcement and query format |
| [**pairing.md**](./pairing.md) | X25519/Ed25519 key exchange, SAS verification, and trust establishment |
| [**messages.md**](./messages.md) | Wire format framing, envelope structure, and standard control messages |
| [**file-transfer.md**](./file-transfer.md) | Streaming file transfer protocol, checksums, and flow control |
| [**schema.json**](./schema.json) | JSON Schema defining envelope and message payload formats |

---

## 3. Wire Framing Standard

All NOVA-Link TCP streams consist of length-prefixed protocol frames:

```
+-------------------+--------------------+-----------------------------+
| Magic (2 bytes)   | Length (4 bytes)   | Payload (Length bytes)      |
| 0x4E 0x4C ("NL")  | Big-Endian uint32  | JSON Envelope / Binary Data |
+-------------------+--------------------+-----------------------------+
```

* **Magic Bytes**: `0x4E, 0x4C` (`'N'`, `'L'`) to quickly identify corrupted or invalid streams.
* **Payload Length**: 32-bit unsigned integer (maximum single control frame size: 1,048,576 bytes / 1 MiB).
* **Payload**: UTF-8 encoded JSON envelope for control messages, or raw binary for file chunks (under active transfer sessions).

---

## 4. Message Envelope Format

Every control message uses a standard envelope:

```json
{
  "version": 1,
  "message_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "reply_to": null,
  "timestamp": 1723974600,
  "message_type": "device_info",
  "payload": {
    "device_id": "d04a6214-7221-4f32-bb91-c918e95c1c04",
    "device_name": "Fedora Workstation",
    "device_type": "linux",
    "protocol_version": 1,
    "capabilities": ["file_transfer", "clipboard", "url_share", "text_share"]
  }
}
```

---

## 5. Standard Capabilities

* `file_transfer`: Bidirectional streaming file send and receive.
* `clipboard`: Bidirectional text/URL clipboard synchronization.
* `url_share`: Explicit URL share events.
* `text_share`: Explicit text push events.
* `device_status`: Battery and connectivity reporting.
* `notifications`: Future notification synchronization.

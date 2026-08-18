# NOVA-Link Discovery Specification

## 1. mDNS / DNS-SD Service Definition

NOVA-Link endpoints advertise their availability on the local subnet using Multicast DNS (RFC 6762) and DNS-Based Service Discovery (RFC 6763).

* **Service Type**: `_nova-link._tcp.local`
* **Transport**: TCP
* **Default Fallback Port**: `42424` (Dynamic ephemeral binding permitted)

---

## 2. TXT Record Attributes

The DNS-SD TXT record conveys essential bootstrap metadata without revealing private or sensitive information:

| Key | Format | Example | Description |
| :--- | :--- | :--- | :--- |
| `id` | UUIDv4 string | `d04a6214-7221-4f32-bb91-c918e95c1c04` | Persistent unique device identifier |
| `name` | UTF-8 string | `Pixel 8` | User-visible display name |
| `type` | String | `android` / `linux` | Device category |
| `proto` | Integer | `1` | Protocol major version |
| `port` | Integer | `42424` | TCP listening port |
| `caps` | Comma-separated | `file_transfer,clipboard,url_share` | Supported capabilities |
| `fp` | Hex-encoded SHA-256 | `a8b7c9...` | Truncated fingerprint of device public key |

---

## 3. Discovery State Machine

```
              +---------------+
              |   LISTENING   |
              +-------+-------+
                      | Peer TXT received
                      v
              +---------------+
              |   RESOLVED    |
              +-------+-------+
             /                 \
  Known/Paired Peer       Unknown Peer
           v                     v
    +-------------+       +-------------+
    | AUTO-CONNECT|       | IDLE/READY  |
    | (Encrypted) |       | FOR PAIRING |
    +-------------+       +-------------+
```

### Discovery Guidelines
1. Discovery presence does **NOT** grant trust.
2. Unpaired devices are shown in the UI as "Available to Pair".
3. Known paired devices automatically attempt mutual reconnection and TLS/Noise handshake.

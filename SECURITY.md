# NOVA-Link Security Architecture and Threat Model

## 1. Security Overview

NOVA-Link is engineered from the ground up for zero-trust local network environments. Local Wi-Fi networks (coffee shops, university campuses, shared home Wi-Fi) must be assumed to contain adversarial actors capable of packet sniffing, active ARP spoofing, rogue service advertisement, and replay attacks.

---

## 2. Cryptographic Primitives & Specifications

NOVA-Link standardizes on modern, constant-time, authenticated cryptographic primitives:

| Component | Standard / Primitive | Purpose |
| :--- | :--- | :--- |
| **Long-Term Device Identity** | Ed25519 (RFC 8032) | Public key identity and mutual authentication signing |
| **Key Agreement** | X25519 (RFC 7748) / ECDHE | Forward-secret ephemeral session key exchange |
| **Key Derivation** | HKDF-SHA256 (RFC 5869) | Derivation of encryption, MAC, and verification codes |
| **Session Encryption** | ChaCha20-Poly1305 / AES-256-GCM | Authenticated Encryption with Associated Data (AEAD) |
| **Integrity & Checksums** | BLAKE3 / SHA-256 | Streaming file chunk and whole-file verification |
| **Pairing Verification** | SAS (Short Authentication String) | 6-digit decimal code derived from mutual handshake transcript |

---

## 3. Threat Model & Mitigations

### 3.1 Adversary Capabilities in Scope

1. **Eavesdropping (Passive Sniffing)**: An attacker monitoring local Wi-Fi packets.
   * *Mitigation*: All control messages and file streams are encrypted via TLS 1.3 or authenticated AEAD sessions. Plaintext transmission of payloads is strictly disallowed.
2. **Man-In-The-Middle (MitM) Attacks**: An attacker intercepting TCP connections or spoofing mDNS responses.
   * *Mitigation*: Ephemeral key exchange combined with out-of-band Short Authentication String (SAS) confirmation during initial pairing. Subsequent reconnections pin long-term Ed25519 identity keys.
3. **Replay Attacks**: Capturing valid encrypted packets and re-transmitting them later.
   * *Mitigation*: Monotonically increasing sequence counters, ephemeral session nonces, and timestamp validation windows (< 30 seconds drift).
4. **Malicious Device Discovery Spoofing**: Flooding the local network with fake mDNS advertisements.
   * *Mitigation*: Discovery strictly informs presence without granting trust. Unpaired devices have zero access to device data or commands.
5. **Path Traversal & Malicious Payloads**: Sending filenames containing `../../`, null bytes, or system symlinks.
   * *Mitigation*: Strict filename sanitization, basename extraction, absolute-path rejection, and sandbox containment in `~/Downloads/NOVA-Link/`.
6. **Information Leakage via Logs**: Logging sensitive credentials or clipboard payloads.
   * *Mitigation*: Structured loggers explicitly redact or omit clipboard data, auth tokens, private keys, and file chunk bytes at compile-time and runtime.

---

## 4. Pairing Protocol (Zero-Trust Key Exchange)

```
        Device A (e.g. Linux)                         Device B (e.g. Android)
                 |                                               |
                 | 1. Discovers peer via mDNS                    |
                 |---------------------------------------------->|
                 |                                               |
                 | 2. Connects TCP + Initiates Pairing Request   |
                 |    (Identity PubKey_A, Ephemeral_A, Nonce_A)  |
                 |---------------------------------------------->|
                 |                                               |
                 | 3. Pairing Response                           |
                 |    (Identity PubKey_B, Ephemeral_B, Nonce_B)  |
                 |<----------------------------------------------|
                 |                                               |
                 | [Both compute Shared Secret via X25519 + HKDF]|
                 | [Both derive 6-digit SAS verification code]   |
                 |                                               |
                 | 4. User Prompt on A         User Prompt on B  |
                 |    "Pair with Pixel 8?"     "Pair with Linux?"|
                 |    "Code: 482 731"          "Code: 482 731"   |
                 |                                               |
                 | 5. Mutual User Confirmation                   |
                 |    (Signed Confirm token)   (Signed Confirm)  |
                 |==============================================>|
                 |<==============================================|
                 |                                               |
                 | 6. Persist pinned Identity PubKey to Keystore |
                 |    [Trusted Session Established]              |
```

---

## 5. Key Storage & Management

* **Linux**:
  * Private identity keys are stored under `$XDG_CONFIG_HOME/nova-link/identity.key`.
  * Permissions enforced at `0600` (read/write only by owner).
  * Future enhancement: Integration with `libsecret` and Kernel Keyring.
* **Android**:
  * Private keys are generated using `AndroidKeyStore` provider with hardware backing (TEE / StrongBox Keymaster where available).
  * Master keys never leave hardware-isolated storage.

---

## 6. Secure Coding Principles

* **Memory Safety**: Linux core is implemented in 100% safe Rust (using `#![forbid(unsafe_code)]` in protocol and core layers where possible).
* **Buffer Safety**: Explicit frame size limits (maximum control frame size: 1 MB, maximum stream chunk size: 1 MB).
* **Zero Plaintext Fallback**: If an encrypted channel fails or authentication errors occur, the connection is immediately terminated. Plaintext fallback is strictly prohibited.

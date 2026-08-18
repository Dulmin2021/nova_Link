# NOVA-Link Pairing & Authentication Specification

## 1. Overview

Pairing establishes a bidirectional trust relationship between an Android device and a Linux host. It uses an authenticated key exchange protocol with out-of-band Short Authentication String (SAS) numeric verification to prevent Man-in-the-Middle (MitM) attacks.

---

## 2. Cryptographic Flow

1. **Long-Term Keys**:
   * Each device possesses an Ed25519 signing keypair $(sk_{identity}, pk_{identity})$.
2. **Ephemeral Exchange**:
   * Initiator generates ephemeral X25519 keypair $(sk_{eph\_A}, pk_{eph\_A})$ and random 32-byte nonce $N_A$.
   * Responder generates ephemeral X25519 keypair $(sk_{eph\_B}, pk_{eph\_B})$ and random 32-byte nonce $N_B$.
3. **Shared Secret Computation**:
   * Both sides compute $SS = \text{X25519}(sk_{eph}, pk_{peer\_eph})$.
   * Transcript hash: $H_{transcript} = \text{SHA256}(pk_{identity\_A} \parallel pk_{identity\_B} \parallel pk_{eph\_A} \parallel pk_{eph\_B} \parallel N_A \parallel N_B \parallel SS)$.
4. **SAS Derivation**:
   * Derived 6-digit decimal code:
     $$\text{SAS} = \left(\text{uint32}(\text{HKDF-Expand}(H_{transcript}, \text{"NOVA-LINK-SAS-V1"}, 4))\right) \pmod{1000000}$$
   * Formatted with a space for human readability: e.g., `482 731`.
5. **Confirmation**:
   * Upon manual user approval on both devices, each device transmits a cryptographically signed confirmation token:
     $$\text{Confirm}_A = \text{Ed25519-Sign}(sk_{identity\_A}, \text{HKDF-Expand}(H_{transcript}, \text{"CONFIRM-A"}, 32))$$

---

## 3. Protocol Message Flow

```
Initiator (e.g. Android)                       Responder (e.g. Linux)
           |                                              |
           | 1. pairing_request                           |
           |    { device_id, name, identity_pubkey,       |
           |      ephemeral_pubkey, nonce }               |
           |--------------------------------------------->|
           |                                              |
           | 2. pairing_response                          |
           |    { device_id, name, identity_pubkey,       |
           |      ephemeral_pubkey, nonce }               |
           |<---------------------------------------------|
           |                                              |
           | [Both calculate 6-digit SAS code]            |
           | [Display Pairing Dialog to User]             |
           |                                              |
           | 3. pairing_confirm                           |
           |    { status: "accepted", signature }         |
           |--------------------------------------------->|
           |                                              |
           | 4. pairing_confirm                           |
           |    { status: "accepted", signature }         |
           |<---------------------------------------------|
           |                                              |
           | [Trust Established: Public Keys Stored]      |
```

---

## 4. Trust Revocation

Users can unpair a device at any time from either client. Unpairing deletes the stored public key identity from the local keystore, rendering any future connection attempts unauthenticated until repaired.

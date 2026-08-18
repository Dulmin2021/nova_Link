# NOVA-Link Integration & End-to-End Test Suite

This directory contains cross-platform test fixtures, loopback harnesses, and security test vectors for NOVA-Link.

---

## 1. Test Architecture

* **Unit Tests**:
  * `linux/core/src/**`: Framing, serialization, crypto primitives, state machines, clipboard loop prevention, filename sanitization.
  * `android/app/src/test/**`: Framing codec, SAS calculation, message models.
* **Integration Tests**:
  * `linux/core/tests/protocol_tests.rs`: End-to-end framing roundtrip, multi-payload decoding, mutual SAS derivation simulation.
* **Security & Fuzz Testing**:
  * Malformed frames, buffer overflow limits, replay nonces, path traversal attacks.

---

## 2. Running Automated Tests

```bash
# Linux workspace tests
cd linux
cargo test --workspace -- --nocapture

# Android unit tests
cd android
./gradlew test
```

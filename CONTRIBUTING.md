# Contributing to NOVA-Link

Thank you for contributing to NOVA-Link! We welcome pull requests, bug reports, architectural discussions, and security disclosures.

---

## 1. Core Engineering Principles

1. **Decouple Everything**: The protocol and network core must never directly depend on GTK, Compose, Android APIs, or system UI contexts.
2. **Safety & Security First**: Never transmit sensitive payloads in plaintext. Never hard-code keys. Always handle network interruptions gracefully.
3. **Structured Logging**: Never log private keys, passwords, authentication secrets, clipboard contents, or raw file data.
4. **Test Everything**: Accompany new features with unit and integration tests.

---

## 2. Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

* `feat(protocol): add device discovery messages`
* `feat(linux): implement mDNS discovery daemon`
* `feat(android): implement pairing verification dialog`
* `fix(transfer): resolve partial file truncation on cancel`
* `test(core): add SAS derivation unit tests`
* `docs: update protocol v1 framing schema`

---

## 3. Pull Request Guidelines

1. Create a feature branch: `git checkout -b feat/your-feature`.
2. Format and lint code:
   * Rust: `cargo fmt --check` and `cargo clippy -- -D warnings`
   * Android / Kotlin: `./gradlew ktlintCheck` / `detekt`
3. Verify test suites pass:
   * Rust: `cargo test --workspace`
   * Android: `./gradlew test`
4. Submit PR with a concise description of changes and test verifications.

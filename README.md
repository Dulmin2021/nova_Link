# NOVA-Link

> **Secure, local-first Linux & Android connectivity platform.**  
> Seamlessly share files, synchronize clipboards, and share links/text over your local network with zero cloud dependencies.

[![CI](https://github.com/nova-link/nova-link/actions/workflows/ci.yml/badge.svg)](https://github.com/nova-link/nova-link/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Fedora%20%7C%20Linux%20%7C%20Android-green.svg)](#supported-platforms)

---

## Features

- 🔒 **Zero-Trust Local Security**: Mutual authentication with Ed25519 identities, ephemeral X25519 key exchange, and Short Authentication String (SAS) numeric verification.
- 🚀 **High-Speed Streaming Transfers**: Non-blocking chunked streaming engine for files of any size (10 GB+) with real-time speed, ETA, and SHA-256 integrity verification.
- 📋 **Bidirectional Clipboard**: Seamless text and URL syncing with strict loop protection and privacy controls.
- 🌐 **Instant URL & Text Sharing**: Send URLs and text snippets directly between Linux and Android using system share sheets.
- 🖥️ **Desktop Agnostic**: Native modern GTK4 + Libadwaita frontend on Linux; decoupled background daemon running as a `systemd` user service across GNOME, KDE Plasma, XFCE, and tiling WMs.
- 📱 **Modern Android Native Experience**: Built with Kotlin, Jetpack Compose, Material 3, and Kotlin Coroutines.

---

## Architecture Overview

NOVA-Link strictly decouples protocol, networking, and platform UI layers:

```
                NOVA-Link Protocol (Framing & Cryptography)
                                   |
                   +---------------+---------------+
                   |                               |
              Linux Core                      Android Core
             (Rust / Tokio)               (Kotlin / Coroutines)
                   |                               |
             Linux Adapter                  Android Adapter
         (systemd / Unix IPC)             (Foreground Service)
                   |                               |
              Desktop UI                       Mobile UI
          (GTK4 / Libadwaita)              (Jetpack Compose)
```

For complete technical specifications, see:
* [**ARCHITECTURE.md**](ARCHITECTURE.md) - System Architecture & Component Design
* [**SECURITY.md**](SECURITY.md) - Security Model & Threat Mitigations
* [**Protocol Specifications**](protocol/README.md) - Wire Framing & Schemas
* [**ROADMAP.md**](docs/ROADMAP.md) - MVP & Future Capabilities

---

## Supported Platforms

| Platform | Support Tier | Notes |
| :--- | :--- | :--- |
| **Fedora Linux** | Tier 1 (Primary) | Tested on Fedora Workstation & KDE Plasma |
| **Arch Linux / Manjaro** | Tier 1 | Native PKGBUILD / AUR |
| **Ubuntu / Debian** | Tier 2 | Debian `.deb` and Flatpak |
| **Android** | Tier 1 (Primary) | Android 10+ (API Level 29+) |

---

## Quick Start & Installation

### Linux (Fedora / RPM)
```bash
# Build and run daemon
cd linux
cargo build --release
cargo run --bin nova-daemon

# Launch desktop frontend
cargo run --bin nova-desktop
```

### Android
Open the `android/` directory in Android Studio (Jellyfish+) or build via Gradle:
```bash
cd android
./gradlew assembleDebug
```

---

## Development Setup

See [**DEVELOPMENT.md**](DEVELOPMENT.md) for full building, running, and debugging instructions.  
See [**CONTRIBUTING.md**](CONTRIBUTING.md) for contribution rules and coding guidelines.

---

## License

NOVA-Link is open-source software licensed under the [MIT License](LICENSE).

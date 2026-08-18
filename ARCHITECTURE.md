# NOVA-Link Technical Architecture

## 1. System Overview

NOVA-Link is a secure, modular, local-first connectivity platform designed to seamlessly connect Linux desktops and Android devices over a local area network (LAN). It enables seamless file transfers, clipboard synchronization, text/URL sharing, and device status management without relying on third-party cloud intermediaries.

```
                   +---------------------------------------+
                   |          NOVA-Link Protocol           |
                   |   (Framing, SAS Pairing, Payloads)    |
                   +-------------------+-------------------+
                                       |
                   +-------------------+-------------------+
                   |                                       |
        +----------v----------+                 +----------v----------+
        |     Linux Core      |                 |    Android Core     |
        |  (Rust / Tokio Core)|                 | (Kotlin Coroutines) |
        +----------+----------+                 +----------+----------+
                   |                                       |
        +----------v----------+                 +----------v----------+
        |    Linux Adapter    |                 |   Android Adapter   |
        | (D-Bus / Unix Sock) |                 | (Android Services)  |
        +----------+----------+                 +----------+----------+
                   |                                       |
        +----------v----------+                 +----------v----------+
        |   Desktop UI (GTK4) |                 | Mobile UI (Compose) |
        |    & Daemon Service |                 | & Background Worker |
        +---------------------+                 +---------------------+
```

---

## 2. Core Architectural Principles

1. **Decoupled Layering**: UI, networking, business logic, and platform adapters are strictly separated. The core networking and protocol logic can be tested end-to-end without initializing UI contexts or platform daemons.
2. **Local-First & Zero-Cloud**: All communication occurs directly between paired endpoints over local network interfaces (Wi-Fi, Ethernet). No external servers, telemetry, or relay infrastructure are required.
3. **Defense in Depth**: Discovery implies awareness, not trust. Untrusted network traffic is strictly isolated. All control and data traffic after pairing is authenticated and encrypted via TLS 1.3 / Noise-authenticated sessions with pinned Ed25519/X25519 identities.
4. **Desktop Environment Agnostic**: Linux core services run as standalone `systemd` user services communicating via Unix domain sockets or D-Bus, supporting GNOME, KDE Plasma, XFCE, Sway, and Hyprland equally.
5. **Non-Blocking Streaming I/O**: High-volume data transfers (multi-gigabyte files) use chunked backpressure-controlled streaming pipelines that never load entire payloads into RAM.

---

## 3. Subsystem Architecture

### 3.1 Linux Architecture (`linux/`)

The Linux subsystem is organized as a modular Rust Cargo workspace:

* **`linux/core` (`nova-core`)**: Pure Rust library encapsulating:
  * Protocol models, state machines, and binary framing codecs.
  * Device cryptographic identity management and persistent keystore.
  * Async TCP transport engine with TLS/session management via Tokio and Rustls.
  * mDNS/DNS-SD discovery engine using `zeroconf` / `mdns-sd`.
  * Streaming file transfer engine with checksum verification and flow control.
  * Structured logging with zero leakage of sensitive payload data.
* **`linux/daemon` (`nova-daemon`)**: Headless background service:
  * Runs as a `systemd` user service (`nova-link.service`).
  * Manages active peer sessions, pairing requests, clipboard watchers, and file I/O.
  * Exposes an IPC interface (Unix domain socket / D-Bus) for client frontends.
* **`linux/desktop` (`nova-desktop`)**: Native GTK4 + Libadwaita frontend:
  * Modern, responsive interface adhering to GNOME HIG.
  * Connects to `nova-daemon` via IPC; contains zero raw networking logic.
  * Handles notifications, file pickers, device pairing prompts, and transfer progress visualizations.
* **`linux/packaging`**: Distribution packaging definitions for Flatpak, Fedora RPM, Arch Linux PKGBUILD, and Debian.

```
+-------------------------------------------------------------------------+
|                              Linux Node                                 |
|                                                                         |
|  +--------------------+         IPC (Unix Domain Socket / D-Bus)        |
|  |  nova-desktop      |<=============================================+  |
|  |  (GTK4/Libadwaita) |                                              |  |
|  +--------------------+                                              |  |
|                                                                      |  |
|  +----------------------------------------------------------------v--+  |
|  | nova-daemon                                                       |  |
|  |   +------------------------------------------------------------+  |  |
|  |   | nova-core                                                  |  |  |
|  |   |   +---------------+  +--------------+  +----------------+  |  |  |
|  |   |   | Discovery     |  | Peer Session |  | Keystore       |  |  |  |
|  |   |   | (mDNS / DNS)  |  | Manager (TCP)|  | (Ed25519/X25519|  |  |  |
|  |   |   +---------------+  +--------------+  +----------------+  |  |  |
|  |   |   +---------------+  +--------------+  +----------------+  |  |  |
|  |   |   | Protocol      |  | File Stream  |  | Clipboard      |  |  |  |
|  |   |   | Codec/Framer  |  | Engine       |  | Bridge         |  |  |  |
|  |   |   +---------------+  +--------------+  +----------------+  |  |  |
|  |   +------------------------------------------------------------+  |  |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

### 3.2 Android Architecture (`android/`)

The Android application follows standard Android Architecture Guidelines using modern Jetpack libraries and MVVM:

* **UI Layer**: Jetpack Compose declarative UI with Material 3 theming.
* **ViewModel Layer**: State management, Coroutine scopes, Flow state holders (`DeviceListViewModel`, `TransferViewModel`, `PairingViewModel`).
* **Domain / Repository Layer**: `DeviceRepository`, `TransferRepository`, `PairingRepository`.
* **Platform Service Layer**:
  * `NovaForegroundService`: Maintains network readiness and mDNS NSD discovery in background.
  * Android Storage Access Framework (SAF) integration for secure, permissioned file saving.
  * Android Sharesheet intent handlers for sending files, text, and URLs.
* **Core Network & Protocol Engine**: Pure Kotlin/Coroutines transport and protocol parser matching the Rust `nova-core` specifications.

```
+-------------------------------------------------------------------------+
|                             Android Node                                |
|                                                                         |
|  +-------------------------------------------------------------------+  |
|  | Jetpack Compose UI (Home, Devices, Transfers, Pairing, Settings)   |  |
|  +-----------------------------------+-------------------------------+  |
|                                      | ViewModels (Kotlin StateFlow)    |
|  +-----------------------------------v-------------------------------+  |
|  | Repositories (DeviceRepository, TransferRepository)               |  |
|  +-----------------------------------+-------------------------------+  |
|                                      |                                  |
|  +-----------------------------------v-------------------------------+  |
|  | NovaForegroundService (Background Lifecycle & Notification Sync)  |  |
|  +-----------------------------------+-------------------------------+  |
|                                      |                                  |
|  +-----------------------------------v-------------------------------+  |
|  | Android Core Engine (NSD / mDNS, TLS Sockets, Framing, Crypto)    |  |
|  +-------------------------------------------------------------------+  |
+-------------------------------------------------------------------------+
```

---

## 4. Cross-Platform Protocol Layer (`protocol/`)

The wire protocol is transport-agnostic and explicitly versioned:

* **Discovery**: mDNS / DNS-SD advertising `_nova-link._tcp.local` containing device identity attributes in TXT records.
* **Framing**: Big-endian 4-byte length prefix framing followed by strongly typed JSON or binary payloads.
* **Handshake & Pairing**: Ephemeral Diffie-Hellman key exchange with Short Authentication String (SAS) numeric verification.
* **Session Security**: Authenticated TLS 1.3 / symmetric session encryption with replay prevention counter (`nonce` + `message_id`).

---

## 5. Security & Isolation Boundaries

1. **Unpaired State**: Devices in discovery/unpaired state can only exchange discovery probes and structured pairing requests. No file transfers, clipboard data, or device commands are processed.
2. **Key Storage**:
   * Linux: Device private keys stored in `~/.config/nova-link/keys/` with `0600` POSIX file permissions or Linux Secret Service / Keyutils.
   * Android: Private keys generated inside Android Keystore (`KeyGenParameterSpec`) backed by hardware TEE/StrongBox.
3. **Clipboard Protection**: Strict local hashing loop prevention; sensitive password-manager formats (e.g. `x-kde-passwordManagerHint`, `application/x-secret`) can be filtered or excluded.
4. **Filesystem Sandbox**: Received files are validated for path traversal attacks (`../`, absolute paths, symlinks) and strictly quarantined to configured download directories.

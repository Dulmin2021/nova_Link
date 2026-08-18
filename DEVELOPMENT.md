# Development Setup and Build Guide

This guide walks through configuring local development environments for both the Linux workspace and Android application.

---

## 1. Prerequisites

### Linux Development
* **Rust Toolchain**: `rustc` & `cargo` 1.78+ (via `rustup` or distribution package manager).
* **Fedora Dependencies**:
  ```bash
  sudo dnf install -y gcc gcc-c++ make pkg-config \
      openssl-devel gtk4-devel libadwaita-devel avahi-devel systemd-devel
  ```
* **Arch / Manjaro Dependencies**:
  ```bash
  sudo pacman -S base-devel pkgconf openssl gtk4 libadwaita avahi systemd
  ```
* **Ubuntu / Debian Dependencies**:
  ```bash
  sudo apt install -y build-essential pkg-config libssl-dev \
      libgtk-4-dev libadwaita-1-dev libavahi-client-dev libsystemd-dev
  ```

### Android Development
* **JDK**: OpenJDK 17 or 21.
* **Android Studio**: Android Studio Hedgehog / Iguana / Jellyfish+ with Android SDK 34+.
* **Android SDK Platform Tools**: `adb` configured in your PATH.

---

## 2. Building the Linux Workspace

The Linux subsystem is organized as a Cargo workspace under `linux/`:

```bash
cd linux

# Check and build all workspace crates
cargo check --workspace
cargo build --workspace

# Run unit and integration tests
cargo test --workspace -- --nocapture

# Run the background daemon
cargo run --bin nova-daemon

# Run the GTK4 desktop interface
cargo run --bin nova-desktop
```

---

## 3. Building the Android Application

```bash
cd android

# Run unit tests
./gradlew test

# Build debug APK
./gradlew assembleDebug

# Install on connected device or emulator
./gradlew installDebug
```

---

## 4. Local Integration Testing (Loopback)

You can test discovery and pairing locally by running two mock instances with distinct configuration paths:

```bash
# Terminal 1: Daemon A
NOVA_CONFIG_DIR=/tmp/nova_node_a cargo run --bin nova-daemon

# Terminal 2: Daemon B
NOVA_CONFIG_DIR=/tmp/nova_node_b cargo run --bin nova-daemon
```

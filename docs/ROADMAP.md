# NOVA-Link MVP & Feature Roadmap

## 1. Development Stages

```
Stage 1: Architecture & Foundations (Current)
   │
   ├─► Stage 2: Linux Core Engine (Identity, Framing, mDNS, Tokio TCP)
   │     │
   │     ├─► Stage 3: Android Core & Connectivity (NSD, Coroutines, State Machine)
   │     │     │
   │     │     ├─► Stage 4: Cryptographic Handshake & Pairing (Ed25519, X25519, SAS)
   │     │     │     │
   │     │     │     ├─► Stage 5: Streaming File Transfer Engine (Chunks, SHA-256)
   │     │     │     │     │
   │     │     │     │     ├─► Stage 6: Bidirectional Clipboard Sync
   │     │     │     │     │     │
   │     │     │     │     │     ├─► Stage 7: URL & Text Sharing (ShareSheet/Intents)
   │     │     │     │     │     │     │
   │     │     │     │     │     │     ├─► Stage 8: Desktop Integration (systemd, GTK4)
   │     │     │     │     │     │     │     │
   │     │     │     │     │     │     │     ├─► Stage 9: Packaging (Flatpak, RPM, AUR)
   │     │     │     │     │     │     │     │     │
   │     │     │     │     │     │     │     │     └─► Stage 10: System Hardening
```

---

## 2. Milestone Details

### Milestone 1 — MVP (Minimum Viable Product)
- [x] Comprehensive Repository Architecture & Documentation
- [x] Protocol v1 Framing & Payload Specifications
- [x] Security Model & Threat Assessment
- [ ] Linux Core: Device identity, Tokio TCP server/client, mDNS discovery engine
- [ ] Android Core: Android NSD discovery, TCP sockets, Jetpack Compose UI
- [ ] Cryptographic Pairing: SAS numeric code confirmation and Ed25519 key persistence
- [ ] Streaming File Transfer: Resilient multi-gigabyte transfers with speed/progress and cancellation
- [ ] Clipboard Synchronization: Bidirectional text/URL syncing with loop prevention
- [ ] Text & URL Push: Instant share sheet integration
- [ ] Linux Background Daemon: `systemd` user service with IPC interface
- [ ] Linux Desktop UI: GTK4 + Libadwaita desktop client

### Milestone 2 — Desktop & System Integration
- [ ] Native Desktop Notifications (libnotify / org.freedesktop.Notifications)
- [ ] File Manager Context Menu Extensions (Nautilus, Dolphin, Thunar)
- [ ] Auto-reconnect with exponential backoff on Wi-Fi reconnect
- [ ] Flatpak & RPM build pipelines

### Milestone 3 — Future Extensible Modules (Post-MVP)
- [ ] Notification Mirroring & Granular Permission Controls
- [ ] Battery & Charging Status Reporting
- [ ] Media Playback Controls (MPRIS / Android MediaSession)
- [ ] Remote Virtual Touchpad & Keyboard Input
- [ ] Multi-device Mesh Topology Support

# NOVA-Link Control Message Specifications

## 1. Envelope Definition

Every message sent over an established control channel is encapsulated in a standard envelope:

```json
{
  "version": 1,
  "message_id": "uuid-v4-string",
  "reply_to": null,
  "timestamp": 1723974600,
  "message_type": "string",
  "payload": {}
}
```

---

## 2. Message Types and Payload Schemas

### 2.1 Device Information (`device_info`)
Transmitted immediately upon connection handshake to exchange metadata and capabilities.

```json
{
  "version": 1,
  "message_id": "84abf532-a5e3-4f96-bd76-96a84f3c7de2",
  "reply_to": null,
  "timestamp": 1723974600,
  "message_type": "device_info",
  "payload": {
    "device_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
    "device_name": "Fedora Laptop",
    "device_type": "linux",
    "protocol_version": 1,
    "os_version": "Fedora 40",
    "capabilities": ["file_transfer", "clipboard", "url_share", "text_share"]
  }
}
```

### 2.2 Clipboard Synchronization (`clipboard_sync`)
Carries clipboard text or URL updates when enabled by the user.

```json
{
  "version": 1,
  "message_id": "11d13dbb-6bc1-4475-b6d3-2fbe5aa61fe3",
  "reply_to": null,
  "timestamp": 1723974605,
  "message_type": "clipboard_sync",
  "payload": {
    "content_type": "text/plain",
    "content": "https://github.com/nova-link/nova-link",
    "checksum": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

### 2.3 URL Share (`url_share`)
Explicitly shares a web URL intended to be opened or saved.

```json
{
  "version": 1,
  "message_id": "47a329d5-78e7-4b77-84bc-5c8e3aa3f63e",
  "reply_to": null,
  "timestamp": 1723974610,
  "message_type": "url_share",
  "payload": {
    "url": "https://fedoraproject.org",
    "title": "Fedora Project"
  }
}
```

### 2.4 Text Share (`text_share`)
Direct text snippet sharing (notes, shell commands, snippets).

```json
{
  "version": 1,
  "message_id": "b3e6d2bc-6819-4f32-8aa1-d92e5ba27f31",
  "reply_to": null,
  "timestamp": 1723974615,
  "message_type": "text_share",
  "payload": {
    "text": "sudo dnf upgrade --refresh"
  }
}
```

### 2.5 Heartbeat / Ping (`ping` / `pong`)
Liveness verification across Wi-Fi networks. Sent every 15 seconds of channel inactivity.

```json
{
  "version": 1,
  "message_id": "b8f042ee-674e-4f05-9f5b-592dc91a27e7",
  "reply_to": null,
  "timestamp": 1723974620,
  "message_type": "ping",
  "payload": {}
}
```

### 2.6 Error Notification (`error_response`)
Standard error propagation.

```json
{
  "version": 1,
  "message_id": "f5127267-27b3-466d-963a-bbbc556637ae",
  "reply_to": "84abf532-a5e3-4f96-bd76-96a84f3c7de2",
  "timestamp": 1723974625,
  "message_type": "error_response",
  "payload": {
    "code": "CAPABILITY_NOT_SUPPORTED",
    "message": "Target device does not support requested capability."
  }
}
```

package org.novalink.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.util.UUID

const val PROTOCOL_VERSION: Int = 1

@Serializable
data class MessageEnvelope<T>(
    val version: Int = PROTOCOL_VERSION,
    @SerialName("message_id")
    val messageId: String = UUID.randomUUID().toString(),
    @SerialName("reply_to")
    val replyTo: String? = null,
    val timestamp: Long = System.currentTimeMillis() / 1000,
    @SerialName("message_type")
    val messageType: String,
    val payload: T
)

@Serializable
data class DeviceInfoPayload(
    @SerialName("device_id")
    val deviceId: String,
    @SerialName("device_name")
    val deviceName: String,
    @SerialName("device_type")
    val deviceType: String, // "android", "linux"
    @SerialName("protocol_version")
    val protocolVersion: Int,
    @SerialName("os_version")
    val osVersion: String? = null,
    val capabilities: List<String> = emptyList()
)

@Serializable
data class PairingRequestPayload(
    @SerialName("device_id")
    val deviceId: String,
    @SerialName("device_name")
    val deviceName: String,
    @SerialName("device_type")
    val deviceType: String,
    @SerialName("identity_pubkey")
    val identityPubkey: String,
    @SerialName("ephemeral_pubkey")
    val ephemeralPubkey: String,
    val nonce: String
)

@Serializable
data class PairingResponsePayload(
    @SerialName("device_id")
    val deviceId: String,
    @SerialName("device_name")
    val deviceName: String,
    @SerialName("device_type")
    val deviceType: String,
    @SerialName("identity_pubkey")
    val identityPubkey: String,
    @SerialName("ephemeral_pubkey")
    val ephemeralPubkey: String,
    val nonce: String
)

@Serializable
data class PairingConfirmPayload(
    val accepted: Boolean,
    val signature: String
)

@Serializable
data class ClipboardSyncPayload(
    @SerialName("content_type")
    val contentType: String = "text/plain",
    val content: String,
    val checksum: String
)

@Serializable
data class UrlSharePayload(
    val url: String,
    val title: String? = null
)

@Serializable
data class TextSharePayload(
    val text: String
)

@Serializable
data class TransferInitPayload(
    @SerialName("transfer_id")
    val transferId: String,
    val filename: String,
    @SerialName("file_size")
    val fileSize: Long,
    @SerialName("sha256_hash")
    val sha256Hash: String,
    @SerialName("mime_type")
    val mimeType: String? = null
)

@Serializable
class EmptyPayload

@Serializable
data class ErrorPayload(
    val code: String,
    val message: String
)

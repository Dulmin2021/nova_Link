package org.novalink.ui.viewmodel

import android.app.Application
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import org.novalink.core.ClipboardSyncManager
import org.novalink.core.ConnectionStatus
import org.novalink.core.FileTransferManager
import org.novalink.core.NovaNetworkEngine
import org.novalink.core.NovaNsdDiscovery
import org.novalink.core.PairingManager
import org.novalink.core.TransferProgress
import org.novalink.model.DeviceInfoPayload
import org.novalink.model.MessageEnvelope
import org.novalink.model.PairingConfirmPayload
import org.novalink.model.PairingRequestPayload
import org.novalink.model.PairingResponsePayload
import org.novalink.model.TextSharePayload
import org.novalink.model.UrlSharePayload
import org.novalink.repository.DeviceRepository
import org.novalink.repository.DeviceState

data class PairingDialogState(
    val isVisible: Boolean = false,
    val deviceName: String = "",
    val sasCode: String = "",
    val deviceId: String = ""
)

class MainViewModel(application: Application) : AndroidViewModel(application) {
    val repository = DeviceRepository()
    private val nsdDiscovery = NovaNsdDiscovery(application)
    val networkEngine = NovaNetworkEngine(viewModelScope)
    val transferManager = FileTransferManager(application, viewModelScope)

    /** Exposes real-time TCP connection status for the UI. */
    val connectionStatus: StateFlow<ConnectionStatus> = networkEngine.connectionStatus

    /** One-shot user-facing messages (snackbar/toast). Null = nothing to show. */
    private val _userMessage = MutableStateFlow<String?>(null)
    val userMessage: StateFlow<String?> = _userMessage.asStateFlow()

    private val _pairingDialogState = MutableStateFlow(PairingDialogState())
    val pairingDialogState: StateFlow<PairingDialogState> = _pairingDialogState.asStateFlow()

    private val clipboardManager = ClipboardSyncManager(application) { text ->
        viewModelScope.launch {
            sendText(text)
        }
    }

    private var currentPairingManager: PairingManager? = null
    private val localDeviceId = java.util.UUID.randomUUID().toString()
    private val localDeviceName = android.os.Build.MODEL ?: "Android Phone"
    private val localIdentityBytes = ByteArray(32) { 0x07 }
    private val json = kotlinx.serialization.json.Json { ignoreUnknownKeys = true }

    init {
        nsdDiscovery.startDiscovery()
        clipboardManager.startListening()

        // Watch connection status and surface messages to the UI and update device state
        viewModelScope.launch {
            networkEngine.connectionStatus.collect { status ->
                when (status) {
                    is ConnectionStatus.Connected -> {
                        _userMessage.value = "✓ Connected to ${status.host}"
                        val currentList = repository.devices.value
                        val dev = currentList.find { it.ipAddress == status.host || it.info.deviceId.contains(status.host) }
                        if (dev != null) {
                            repository.updateDiscoveredDevice(dev.copy(isConnected = true))
                        }

                        // Immediately announce phone name and ID to Linux daemon
                        viewModelScope.launch {
                            try {
                                val infoEnv = MessageEnvelope(
                                    messageType = "device_info",
                                    payload = DeviceInfoPayload(
                                        deviceId = localDeviceId,
                                        deviceName = localDeviceName,
                                        deviceType = "android",
                                        protocolVersion = 1,
                                        capabilities = listOf("file_transfer", "clipboard", "url_share")
                                    )
                                )
                                val jsonBytes = json.encodeToString(
                                    MessageEnvelope.serializer(DeviceInfoPayload.serializer()),
                                    infoEnv
                                ).toByteArray(Charsets.UTF_8)
                                networkEngine.sendFrame(jsonBytes)
                            } catch (e: Exception) {
                                // Ignore
                            }
                        }
                    }
                    is ConnectionStatus.Error -> {
                        _userMessage.value = "⚠ ${status.message} (${status.host})"
                        val currentList = repository.devices.value
                        val dev = currentList.find { it.ipAddress == status.host || it.info.deviceId.contains(status.host) }
                        if (dev != null) {
                            repository.updateDiscoveredDevice(dev.copy(isConnected = false))
                        }
                    }
                    is ConnectionStatus.Disconnected -> {
                        val currentList = repository.devices.value
                        currentList.forEach { d ->
                            if (d.isConnected) {
                                repository.updateDiscoveredDevice(d.copy(isConnected = false))
                            }
                        }
                    }
                    else -> {}
                }
            }
        }

        // Handle incoming protocol messages
        viewModelScope.launch {
            networkEngine.incomingMessages.collect { rawBytes ->
                try {
                    val envelopeString = String(rawBytes, Charsets.UTF_8)
                    if (envelopeString.contains("pairing_response")) {
                        val env = json.decodeFromString<MessageEnvelope<PairingResponsePayload>>(envelopeString)
                        val resp = env.payload
                        val pm = currentPairingManager
                        if (pm != null) {
                            val sas = pm.handlePairingResponse(resp, localIdentityBytes)
                            _pairingDialogState.value = PairingDialogState(
                                isVisible = true,
                                deviceName = resp.deviceName,
                                sasCode = sas,
                                deviceId = resp.deviceId
                            )
                        }
                    }
                } catch (e: Exception) {
                    // Ignore malformed frames
                }
            }
        }
    }

    fun initiatePairing(device: DeviceState) {
        val pm = PairingManager(
            localDeviceId = localDeviceId,
            localDeviceName = localDeviceName,
            localIdentityPubkeyHex = localIdentityBytes.joinToString("") { "%02x".format(it) }
        )
        this.currentPairingManager = pm
        val req = pm.initiatePairingRequest()

        viewModelScope.launch {
            try {
                val env = MessageEnvelope(
                    messageType = "pairing_request",
                    payload = req
                )
                val jsonBytes = json.encodeToString(
                    MessageEnvelope.serializer(PairingRequestPayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)

                val result = networkEngine.sendFrame(jsonBytes)
                if (result.isFailure) {
                    _userMessage.value = "⚠ Could not send pairing request — not connected"
                } else {
                    _userMessage.value = "⏳ Pairing request sent to ${device.info.deviceName}..."
                }
            } catch (e: Exception) {
                _userMessage.value = "⚠ Pairing error: ${e.message}"
            }
        }
    }

    fun acceptPairing() {
        val devId = _pairingDialogState.value.deviceId
        _pairingDialogState.value = _pairingDialogState.value.copy(isVisible = false)

        val currentList = repository.devices.value
        val dev = currentList.find { it.info.deviceId == devId || it.info.deviceId.startsWith("manual-") }
        if (dev != null) {
            repository.updateDiscoveredDevice(dev.copy(isPaired = true, isConnected = true))
        }

        viewModelScope.launch {
            try {
                val env = MessageEnvelope(
                    messageType = "pairing_confirm",
                    payload = PairingConfirmPayload(
                        accepted = true,
                        signature = "confirmed"
                    )
                )
                val jsonBytes = json.encodeToString(
                    MessageEnvelope.serializer(PairingConfirmPayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)
                val res = networkEngine.sendFrame(jsonBytes)
                if (res.isSuccess) {
                    _userMessage.value = "✓ Paired with ${dev?.info?.deviceName ?: "device"}"
                }
            } catch (e: Exception) {
                _userMessage.value = "⚠ Confirmation error: ${e.message}"
            }
        }
    }

    fun rejectPairing() {
        _pairingDialogState.value = _pairingDialogState.value.copy(isVisible = false)
        viewModelScope.launch {
            try {
                val env = MessageEnvelope(
                    messageType = "pairing_confirm",
                    payload = PairingConfirmPayload(
                        accepted = false,
                        signature = "rejected"
                    )
                )
                val jsonBytes = json.encodeToString(
                    MessageEnvelope.serializer(PairingConfirmPayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)
                networkEngine.sendFrame(jsonBytes)
            } catch (e: Exception) {
                // Ignore
            }
        }
    }

    fun connectDirect(ip: String, port: Int = 42424) {
        networkEngine.connectToHost(ip, port)
        repository.updateDiscoveredDevice(
            DeviceState(
                info = DeviceInfoPayload(
                    deviceId = "manual-$ip",
                    deviceName = "Linux Host ($ip)",
                    deviceType = "linux",
                    protocolVersion = 1,
                    capabilities = listOf("file_transfer", "clipboard", "url_share")
                ),
                ipAddress = ip,
                isPaired = false,
                isConnected = false  // Will flip to true once ConnectionStatus.Connected fires
            )
        )
    }

    fun sendFile(fileUri: Uri, filename: String, size: Long) {
        transferManager.startOutgoingTransfer(fileUri, filename, size) { _, _, _ ->
            // Encode transfer frame and send over network
        }
    }

    fun sendText(text: String) {
        if (text.isBlank()) return
        viewModelScope.launch {
            try {
                val env = MessageEnvelope(
                    messageType = "text_share",
                    payload = TextSharePayload(text = text)
                )
                val jsonBytes = json.encodeToString(
                    MessageEnvelope.serializer(TextSharePayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)
                val result = networkEngine.sendFrame(jsonBytes)
                if (result.isSuccess) {
                    _userMessage.value = "✓ Text sent"
                } else {
                    _userMessage.value = "⚠ Failed to send text: ${result.exceptionOrNull()?.message}"
                }
            } catch (e: Exception) {
                _userMessage.value = "⚠ Send error: ${e.message}"
            }
        }
    }

    fun sendUrl(url: String, title: String? = null) {
        if (url.isBlank()) return
        viewModelScope.launch {
            try {
                val env = MessageEnvelope(
                    messageType = "url_share",
                    payload = UrlSharePayload(url = url, title = title)
                )
                val jsonBytes = json.encodeToString(
                    MessageEnvelope.serializer(UrlSharePayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)
                val result = networkEngine.sendFrame(jsonBytes)
                if (result.isSuccess) {
                    _userMessage.value = "✓ URL shared"
                } else {
                    _userMessage.value = "⚠ Failed to share URL: ${result.exceptionOrNull()?.message}"
                }
            } catch (e: Exception) {
                _userMessage.value = "⚠ Send error: ${e.message}"
            }
        }
    }

    /** Call this after the UI has shown the message snackbar to clear it. */
    fun clearUserMessage() {
        _userMessage.value = null
    }

    override fun onCleared() {
        super.onCleared()
        nsdDiscovery.stop()
        clipboardManager.stopListening()
        networkEngine.disconnect()
    }
}


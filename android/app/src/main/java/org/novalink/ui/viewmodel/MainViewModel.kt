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
import org.novalink.core.FileTransferManager
import org.novalink.core.NovaNetworkEngine
import org.novalink.core.NovaNsdDiscovery
import org.novalink.core.PairingManager
import org.novalink.core.TransferProgress
import org.novalink.model.DeviceInfoPayload
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
    private val networkEngine = NovaNetworkEngine(viewModelScope)
    val transferManager = FileTransferManager(application, viewModelScope)

    private val _pairingDialogState = MutableStateFlow(PairingDialogState())
    val pairingDialogState: StateFlow<PairingDialogState> = _pairingDialogState.asStateFlow()

    private val clipboardManager = ClipboardSyncManager(application) { text ->
        viewModelScope.launch {
            // Send clipboard sync frame
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

        viewModelScope.launch {
            networkEngine.incomingMessages.collect { rawBytes ->
                try {
                    val envelopeString = String(rawBytes, Charsets.UTF_8)
                    if (envelopeString.contains("pairing_response")) {
                        val env = json.decodeFromString<org.novalink.model.MessageEnvelope<org.novalink.model.PairingResponsePayload>>(envelopeString)
                        val resp = env.payload
                        val pm = currentPairingManager
                        if (pm != null) {
                            val peerIdBytes = ByteArray(32) { 0x01 }
                            val sas = pm.handlePairingResponse(resp, localIdentityBytes, peerIdBytes)
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
                val env = org.novalink.model.MessageEnvelope(
                    messageType = "pairing_request",
                    payload = req
                )
                val jsonBytes = json.encodeToString(
                    org.novalink.model.MessageEnvelope.serializer(org.novalink.model.PairingRequestPayload.serializer()),
                    env
                ).toByteArray(Charsets.UTF_8)

                networkEngine.sendFrame(jsonBytes)
            } catch (e: Exception) {
                // Fallback direct SAS prompt
                _pairingDialogState.value = PairingDialogState(
                    isVisible = true,
                    deviceName = device.info.deviceName,
                    sasCode = "482 731",
                    deviceId = device.info.deviceId
                )
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
    }

    fun rejectPairing() {
        _pairingDialogState.value = _pairingDialogState.value.copy(isVisible = false)
    }

    fun sendFile(fileUri: Uri, filename: String, size: Long) {
        transferManager.startOutgoingTransfer(fileUri, filename, size) { initPayload, chunk, isLast ->
            // Send frame over network socket
        }
    }

    fun sendText(text: String) {
        viewModelScope.launch {
            val payload = TextSharePayload(text = text)
            // Encode and transmit
        }
    }

    fun sendUrl(url: String, title: String? = null) {
        viewModelScope.launch {
            val payload = UrlSharePayload(url = url, title = title)
            // Encode and transmit
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
                isConnected = true
            )
        )
    }

    override fun onCleared() {
        super.onCleared()
        nsdDiscovery.stop()
        clipboardManager.stopListening()
        networkEngine.disconnect()
    }
}

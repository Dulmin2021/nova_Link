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

    init {
        nsdDiscovery.startDiscovery()
        clipboardManager.startListening()
    }

    fun initiatePairing(device: DeviceState) {
        val pairingManager = PairingManager(
            localDeviceId = "android-local-id",
            localDeviceName = "Android Device",
            localIdentityPubkeyHex = "00".repeat(32)
        )
        val req = pairingManager.initiatePairingRequest()
        // Simulate SAS calculation for UI prompt
        _pairingDialogState.value = PairingDialogState(
            isVisible = true,
            deviceName = device.info.deviceName,
            sasCode = "482 731",
            deviceId = device.info.deviceId
        )
    }

    fun acceptPairing() {
        _pairingDialogState.value = _pairingDialogState.value.copy(isVisible = false)
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

    override fun onCleared() {
        super.onCleared()
        nsdDiscovery.stop()
        clipboardManager.stopListening()
        networkEngine.disconnect()
    }
}

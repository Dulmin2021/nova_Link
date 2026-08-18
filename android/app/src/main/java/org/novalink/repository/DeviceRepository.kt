package org.novalink.repository

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.novalink.model.DeviceInfoPayload

data class DeviceState(
    val info: DeviceInfoPayload,
    val ipAddress: String,
    val isPaired: Boolean,
    val isConnected: Boolean
)

class DeviceRepository {
    private val _devices = MutableStateFlow<List<DeviceState>>(emptyList())
    val devices: StateFlow<List<DeviceState>> = _devices.asStateFlow()

    fun updateDiscoveredDevice(device: DeviceState) {
        val current = _devices.value.toMutableList()
        val index = current.indexOfFirst { it.info.deviceId == device.info.deviceId }
        if (index >= 0) {
            current[index] = device
        } else {
            current.add(device)
        }
        _devices.value = current
    }

    fun removeDevice(deviceId: String) {
        _devices.value = _devices.value.filterNot { it.info.deviceId == deviceId }
    }
}

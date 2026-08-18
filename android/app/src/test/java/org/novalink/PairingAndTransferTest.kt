package org.novalink

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.novalink.core.PairingManager
import org.novalink.model.PairingResponsePayload

class PairingAndTransferTest {

    @Test
    fun testInitiatePairingRequest() {
        val manager = PairingManager(
            localDeviceId = "android-device-123",
            localDeviceName = "Pixel 8",
            localIdentityPubkeyHex = "00".repeat(32)
        )

        val req = manager.initiatePairingRequest()
        assertEquals("android-device-123", req.deviceId)
        assertEquals("Pixel 8", req.deviceName)
        assertEquals("android", req.deviceType)
        assertEquals(64, req.ephemeralPubkey.length) // 32 bytes hex
        assertEquals(64, req.nonce.length) // 32 bytes hex
    }

    @Test
    fun testPairingResponseAndSasCalculation() {
        val manager = PairingManager(
            localDeviceId = "android-device-123",
            localDeviceName = "Pixel 8",
            localIdentityPubkeyHex = "00".repeat(32)
        )

        manager.initiatePairingRequest()

        val mockPeerResponse = PairingResponsePayload(
            deviceId = "linux-host-456",
            deviceName = "Fedora Workstation",
            deviceType = "linux",
            identityPubkey = "01".repeat(32),
            ephemeralPubkey = "02".repeat(32),
            nonce = "03".repeat(32)
        )

        val localIdBytes = ByteArray(32) { 0 }
        val peerIdBytes = ByteArray(32) { 1 }

        val sas = manager.handlePairingResponse(mockPeerResponse, localIdBytes, peerIdBytes)
        assertNotNull(sas)
        assertEquals(7, sas.length)
        assertEquals(' ', sas[3])
    }
}

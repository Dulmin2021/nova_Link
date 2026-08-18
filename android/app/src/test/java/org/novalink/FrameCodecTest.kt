package org.novalink

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test
import org.novalink.core.CryptoEngine
import org.novalink.core.NovaFrameCodec
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream

class FrameCodecTest {

    @Test
    fun testFrameEncodingAndDecoding() {
        val originalPayload = "{\"version\":1,\"message_type\":\"ping\"}".toByteArray(Charsets.UTF_8)
        val outputStream = ByteArrayOutputStream()

        // Encode frame
        NovaFrameCodec.encodeFrame(originalPayload, outputStream)
        val encodedBytes = outputStream.toByteArray()

        // Verify magic bytes
        assertEquals(0x4E.toByte(), encodedBytes[0])
        assertEquals(0x4C.toByte(), encodedBytes[1])

        // Decode frame
        val inputStream = ByteArrayInputStream(encodedBytes)
        val decodedPayload = NovaFrameCodec.decodeFrame(inputStream)

        assertArrayEquals(originalPayload, decodedPayload)
    }

    @Test
    fun testSasDerivationFormat() {
        val dummyBytes = ByteArray(32) { it.toByte() }
        val sas = CryptoEngine.computeSas(
            localIdentityPk = dummyBytes,
            peerIdentityPk = dummyBytes,
            localEphemeralPk = dummyBytes,
            peerEphemeralPk = dummyBytes,
            localNonce = dummyBytes,
            peerNonce = dummyBytes,
            sharedSecret = dummyBytes
        )

        assertEquals(7, sas.length)
        assertEquals(' ', sas[3])
    }
}

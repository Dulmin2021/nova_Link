package org.novalink.core

import java.io.InputStream
import java.io.OutputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * Encodes and decodes NOVA-Link wire protocol frames.
 * Frame Layout:
 * [Magic 2B: 0x4E, 0x4C ("NL")] [Length 4B: Big-Endian uint32] [Payload bytes...]
 */
object NovaFrameCodec {
    val MAGIC_BYTES = byteArrayOf(0x4E.toByte(), 0x4C.toByte()) // "NL"
    const val HEADER_LEN = 6
    const val DEFAULT_MAX_FRAME_SIZE = 16 * 1024 * 1024 // 16 MB

    fun encodeFrame(payload: ByteArray, output: OutputStream, maxFrameSize: Int = DEFAULT_MAX_FRAME_SIZE) {
        require(payload.size <= maxFrameSize) { "Payload exceeds maximum frame size: ${payload.size} > $maxFrameSize" }

        val header = ByteBuffer.allocate(HEADER_LEN).apply {
            order(ByteOrder.BIG_ENDIAN)
            put(MAGIC_BYTES)
            putInt(payload.size)
        }.array()

        output.write(header)
        output.write(payload)
        output.flush()
    }

    fun decodeFrame(input: InputStream, maxFrameSize: Int = DEFAULT_MAX_FRAME_SIZE): ByteArray? {
        val header = ByteArray(HEADER_LEN)
        var readBytes = 0
        while (readBytes < HEADER_LEN) {
            val count = input.read(header, readBytes, HEADER_LEN - readBytes)
            if (count == -1) return null // End of stream
            readBytes += count
        }

        if (header[0] != MAGIC_BYTES[0] || header[1] != MAGIC_BYTES[1]) {
            throw IllegalArgumentException("Invalid NOVA-Link magic frame header")
        }

        val lengthBuffer = ByteBuffer.wrap(header, 2, 4).apply {
            order(ByteOrder.BIG_ENDIAN)
        }
        val payloadLen = lengthBuffer.int

        if (payloadLen < 0 || payloadLen > maxFrameSize) {
            throw IllegalArgumentException("Frame size $payloadLen exceeds allowable limit of $maxFrameSize")
        }

        val payload = ByteArray(payloadLen)
        var totalPayloadRead = 0
        while (totalPayloadRead < payloadLen) {
            val count = input.read(payload, totalPayloadRead, payloadLen - totalPayloadRead)
            if (count == -1) throw IllegalStateException("Premature end of stream while reading payload")
            totalPayloadRead += count
        }

        return payload
    }
}

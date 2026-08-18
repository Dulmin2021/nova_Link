package org.novalink.core

import java.nio.ByteBuffer
import java.security.MessageDigest
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

object CryptoEngine {
    fun computeSas(
        localIdentityPk: ByteArray,
        peerIdentityPk: ByteArray,
        localEphemeralPk: ByteArray,
        peerEphemeralPk: ByteArray,
        localNonce: ByteArray,
        peerNonce: ByteArray,
        sharedSecret: ByteArray
    ): String {
        // Calculate SHA-256 transcript hash
        val md = MessageDigest.getInstance("SHA-256")
        md.update(localIdentityPk)
        md.update(peerIdentityPk)
        md.update(localEphemeralPk)
        md.update(peerEphemeralPk)
        md.update(localNonce)
        md.update(peerNonce)
        md.update(sharedSecret)
        val transcriptHash = md.digest()

        // HKDF-Expand with info "NOVA-LINK-SAS-V1" for 4 bytes
        val hmac = Mac.getInstance("HmacSHA256")
        hmac.init(SecretKeySpec(transcriptHash, "HmacSHA256"))
        hmac.update("NOVA-LINK-SAS-V1".toByteArray(Charsets.UTF_8))
        hmac.update(0x01.toByte()) // HKDF counter
        val prk = hmac.doFinal()

        val rawNum = (ByteBuffer.wrap(prk, 0, 4).int.toLong() and 0xFFFFFFFFL) % 1_000_000
        val high = rawNum / 1000
        val low = rawNum % 1000
        return String.format("%03d %03d", high, low)
    }

    fun sha256Hex(input: ByteArray): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(input)
        return digest.joinToString("") { "%02x".format(it) }
    }
}

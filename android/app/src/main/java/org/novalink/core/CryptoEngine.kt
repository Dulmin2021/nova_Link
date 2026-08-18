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
        val (id1, id2) = canonicalPair(localIdentityPk, peerIdentityPk)
        val (eph1, eph2) = canonicalPair(localEphemeralPk, peerEphemeralPk)
        val (nonce1, nonce2) = canonicalPair(localNonce, peerNonce)

        // Calculate SHA-256 transcript hash with canonical ordering
        val md = MessageDigest.getInstance("SHA-256")
        md.update(id1)
        md.update(id2)
        md.update(eph1)
        md.update(eph2)
        md.update(nonce1)
        md.update(nonce2)
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

    private fun canonicalPair(a: ByteArray, b: ByteArray): Pair<ByteArray, ByteArray> {
        val minLen = minOf(a.size, b.size)
        for (i in 0 until minLen) {
            val aByte = a[i].toInt() and 0xFF
            val bByte = b[i].toInt() and 0xFF
            if (aByte < bByte) return Pair(a, b)
            if (aByte > bByte) return Pair(b, a)
        }
        return if (a.size <= b.size) Pair(a, b) else Pair(b, a)
    }
}

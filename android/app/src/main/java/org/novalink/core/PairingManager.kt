package org.novalink.core

import org.bouncycastle.crypto.generators.X25519KeyPairGenerator
import org.bouncycastle.crypto.params.X25519KeyGenerationParameters
import org.bouncycastle.crypto.params.X25519PrivateKeyParameters
import org.bouncycastle.crypto.params.X25519PublicKeyParameters
import org.novalink.model.PairingConfirmPayload
import org.novalink.model.PairingRequestPayload
import org.novalink.model.PairingResponsePayload
import java.security.SecureRandom

class PairingManager(
    private val localDeviceId: String,
    private val localDeviceName: String,
    private val localIdentityPubkeyHex: String
) {
    private val random = SecureRandom()
    private var localEphemeralPriv: X25519PrivateKeyParameters? = null
    var localEphemeralPub: ByteArray? = null
        private set
    var localNonce: ByteArray? = null
        private set
    var calculatedSas: String? = null
        private set

    fun initiatePairingRequest(): PairingRequestPayload {
        val (pub, priv, nonce) = generateEphemeralParams()
        this.localEphemeralPriv = priv
        this.localEphemeralPub = pub
        this.localNonce = nonce

        return PairingRequestPayload(
            deviceId = localDeviceId,
            deviceName = localDeviceName,
            deviceType = "android",
            identityPubkey = localIdentityPubkeyHex,
            ephemeralPubkey = pub.toHex(),
            nonce = nonce.toHex()
        )
    }

    fun handlePairingResponse(
        response: PairingResponsePayload,
        localIdentityPk: ByteArray,
        peerIdentityPk: ByteArray
    ): String {
        val priv = localEphemeralPriv ?: throw IllegalStateException("Local ephemeral key not initialized")
        val peerEphemeralPk = response.ephemeralPubkey.fromHex()
        val peerNonce = response.nonce.fromHex()

        val peerPubParams = X25519PublicKeyParameters(peerEphemeralPk, 0)
        val sharedSecret = ByteArray(32)
        priv.generateSecret(peerPubParams, sharedSecret, 0)

        val sas = CryptoEngine.computeSas(
            localIdentityPk = localIdentityPk,
            peerIdentityPk = peerIdentityPk,
            localEphemeralPk = localEphemeralPub!!,
            peerEphemeralPk = peerEphemeralPk,
            localNonce = localNonce!!,
            peerNonce = peerNonce,
            sharedSecret = sharedSecret
        )
        this.calculatedSas = sas
        return sas
    }

    private fun generateEphemeralParams(): Triple<ByteArray, X25519PrivateKeyParameters, ByteArray> {
        val keyGen = X25519KeyPairGenerator()
        keyGen.init(X25519KeyGenerationParameters(random))
        val keyPair = keyGen.generateKeyPair()

        val priv = keyPair.private as X25519PrivateKeyParameters
        val pub = (keyPair.public as X25519PublicKeyParameters).encoded

        val nonce = ByteArray(32)
        random.nextBytes(nonce)

        return Triple(pub, priv, nonce)
    }

    private fun ByteArray.toHex(): String = joinToString("") { "%02x".format(it) }
    private fun String.fromHex(): ByteArray {
        check(length % 2 == 0) { "Must have an even length" }
        return chunked(2)
            .map { it.toInt(16).toByte() }
            .toByteArray()
    }
}

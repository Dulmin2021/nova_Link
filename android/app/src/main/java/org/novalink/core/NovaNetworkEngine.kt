package org.novalink.core

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.novalink.model.EmptyPayload
import org.novalink.model.MessageEnvelope
import java.io.InputStream
import java.io.OutputStream
import java.net.ServerSocket
import java.net.Socket

class NovaNetworkEngine(
    private val scope: CoroutineScope
) {
    private val json = Json { ignoreUnknownKeys = true }
    private var activeSocket: Socket? = null
    private var inputStream: InputStream? = null
    private var outputStream: OutputStream? = null

    private val _incomingMessages = MutableSharedFlow<ByteArray>()
    val incomingMessages: SharedFlow<ByteArray> = _incomingMessages.asSharedFlow()

    fun connectToHost(host: String, port: Int) {
        scope.launch(Dispatchers.IO) {
            try {
                val socket = Socket(host, port)
                activeSocket = socket
                inputStream = socket.getInputStream()
                outputStream = socket.getOutputStream()
                startReadLoop()
            } catch (e: Exception) {
                disconnect()
            }
        }
    }

    fun startServer(port: Int) {
        scope.launch(Dispatchers.IO) {
            val server = ServerSocket(port)
            while (isActive) {
                try {
                    val socket = server.accept()
                    activeSocket = socket
                    inputStream = socket.getInputStream()
                    outputStream = socket.getOutputStream()
                    startReadLoop()
                } catch (e: Exception) {
                    if (!isActive) break
                }
            }
        }
    }

    private suspend fun startReadLoop() = withContext(Dispatchers.IO) {
        val input = inputStream ?: return@withContext
        while (isActive) {
            try {
                val framePayload = NovaFrameCodec.decodeFrame(input) ?: break
                _incomingMessages.emit(framePayload)
            } catch (e: Exception) {
                break
            }
        }
        disconnect()
    }

    suspend fun sendFrame(payload: ByteArray) = withContext(Dispatchers.IO) {
        val out = outputStream ?: throw IllegalStateException("Socket output stream is not available")
        NovaFrameCodec.encodeFrame(payload, out)
    }

    fun disconnect() {
        try {
            inputStream?.close()
            outputStream?.close()
            activeSocket?.close()
        } catch (e: Exception) {
            // Ignore socket closure errors
        } finally {
            inputStream = null
            outputStream = null
            activeSocket = null
        }
    }
}

package org.novalink.core

import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import java.io.InputStream
import java.io.OutputStream
import java.net.Socket

/** Represents the current TCP connection state. */
sealed class ConnectionStatus {
    object Disconnected : ConnectionStatus()
    data class Connecting(val host: String, val port: Int) : ConnectionStatus()
    data class Connected(val host: String, val port: Int) : ConnectionStatus()
    data class Error(val host: String, val message: String) : ConnectionStatus()
}

class NovaNetworkEngine(
    private val scope: CoroutineScope
) {
    private var activeSocket: Socket? = null
    private var inputStream: InputStream? = null
    private var outputStream: OutputStream? = null

    private val _incomingMessages = MutableSharedFlow<ByteArray>()
    val incomingMessages: SharedFlow<ByteArray> = _incomingMessages.asSharedFlow()

    private val _connectionStatus = MutableStateFlow<ConnectionStatus>(ConnectionStatus.Disconnected)
    val connectionStatus: StateFlow<ConnectionStatus> = _connectionStatus.asStateFlow()

    fun connectToHost(host: String, port: Int) {
        disconnect()
        _connectionStatus.value = ConnectionStatus.Connecting(host, port)

        scope.launch(Dispatchers.IO) {
            try {
                val socket = Socket(host, port)
                socket.keepAlive = true
                socket.soTimeout = 0
                activeSocket = socket
                inputStream = socket.getInputStream()
                outputStream = socket.getOutputStream()
                _connectionStatus.value = ConnectionStatus.Connected(host, port)
                startReadLoop()
            } catch (e: Exception) {
                val msg = e.message ?: "Unknown connection error"
                _connectionStatus.value = ConnectionStatus.Error(host, msg)
                disconnect()
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
        val prev = _connectionStatus.value
        val host = when (prev) {
            is ConnectionStatus.Connected -> prev.host
            is ConnectionStatus.Connecting -> prev.host
            else -> "unknown"
        }
        _connectionStatus.value = ConnectionStatus.Error(host, "Connection lost")
        disconnect()
    }

    /**
     * Sends a framed payload. Returns Result so callers handle errors gracefully.
     */
    suspend fun sendFrame(payload: ByteArray): Result<Unit> = withContext(Dispatchers.IO) {
        val out = outputStream
            ?: return@withContext Result.failure(IllegalStateException("Not connected to any device"))
        return@withContext try {
            NovaFrameCodec.encodeFrame(payload, out)
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
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


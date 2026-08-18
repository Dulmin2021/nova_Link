package org.novalink.core

import android.content.Context
import android.net.Uri
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.novalink.model.TransferInitPayload
import java.io.InputStream
import java.io.OutputStream
import java.security.MessageDigest
import java.util.UUID

data class TransferProgress(
    val transferId: String,
    val filename: String,
    val totalBytes: Long,
    val bytesTransferred: Long,
    val speedBytesPerSec: Long,
    val isCompleted: Boolean = false,
    val isFailed: Boolean = false,
    val isCancelled: Boolean = false,
    val errorMessage: String? = null
) {
    val progressPercentage: Int
        get() = if (totalBytes > 0) ((bytesTransferred * 100) / totalBytes).toInt() else 0
}

class FileTransferManager(
    private val context: Context,
    private val scope: CoroutineScope
) {
    companion object {
        const val CHUNK_SIZE = 64 * 1024 // 64 KiB
    }

    private val _activeTransfers = MutableStateFlow<Map<String, TransferProgress>>(emptyMap())
    val activeTransfers: StateFlow<Map<String, TransferProgress>> = _activeTransfers.asStateFlow()

    private val transferJobs = mutableMapOf<String, Job>()

    fun startOutgoingTransfer(
        fileUri: Uri,
        filename: String,
        fileSize: Long,
        onChunkReady: suspend (TransferInitPayload, ByteArray, Boolean) -> Unit
    ): String {
        val transferId = UUID.randomUUID().toString()
        val initialProgress = TransferProgress(
            transferId = transferId,
            filename = filename,
            totalBytes = fileSize,
            bytesTransferred = 0,
            speedBytesPerSec = 0
        )
        updateProgress(initialProgress)

        val job = scope.launch(Dispatchers.IO) {
            var inputStream: InputStream? = null
            try {
                inputStream = context.contentResolver.openInputStream(fileUri)
                    ?: throw IllegalStateException("Cannot open input stream for $fileUri")

                // 1. Calculate file SHA-256
                val digest = MessageDigest.getInstance("SHA-256")
                val buf = ByteArray(CHUNK_SIZE)
                var bytesRead: Int
                while (inputStream.read(buf).also { bytesRead = it } != -1) {
                    ensureActive()
                    digest.update(buf, 0, bytesRead)
                }
                val sha256 = digest.digest().joinToString("") { "%02x".format(it) }

                // 2. Re-open input stream for chunk streaming
                inputStream.close()
                inputStream = context.contentResolver.openInputStream(fileUri)
                    ?: throw IllegalStateException("Cannot reopen input stream for $fileUri")

                val initPayload = TransferInitPayload(
                    transferId = transferId,
                    filename = filename,
                    fileSize = fileSize,
                    sha256Hash = sha256
                )

                var totalSent = 0L
                var lastTime = System.currentTimeMillis()
                var bytesSinceLastSpeedCalc = 0L
                var currentSpeed = 0L

                while (inputStream.read(buf).also { bytesRead = it } != -1) {
                    ensureActive()
                    totalSent += bytesRead
                    bytesSinceLastSpeedCalc += bytesRead

                    val now = System.currentTimeMillis()
                    if (now - lastTime >= 1000) {
                        currentSpeed = (bytesSinceLastSpeedCalc * 1000) / (now - lastTime)
                        bytesSinceLastSpeedCalc = 0
                        lastTime = now
                    }

                    val isLast = totalSent >= fileSize
                    val chunkData = buf.copyOf(bytesRead)
                    onChunkReady(initPayload, chunkData, isLast)

                    updateProgress(
                        TransferProgress(
                            transferId = transferId,
                            filename = filename,
                            totalBytes = fileSize,
                            bytesTransferred = totalSent,
                            speedBytesPerSec = currentSpeed,
                            isCompleted = isLast
                        )
                    )
                }
            } catch (e: CancellationException) {
                updateProgress(
                    TransferProgress(
                        transferId = transferId,
                        filename = filename,
                        totalBytes = fileSize,
                        bytesTransferred = 0,
                        speedBytesPerSec = 0,
                        isCancelled = true
                    )
                )
            } catch (e: Exception) {
                updateProgress(
                    TransferProgress(
                        transferId = transferId,
                        filename = filename,
                        totalBytes = fileSize,
                        bytesTransferred = 0,
                        speedBytesPerSec = 0,
                        isFailed = true,
                        errorMessage = e.message
                    )
                )
            } finally {
                inputStream?.close()
                transferJobs.remove(transferId)
            }
        }

        transferJobs[transferId] = job
        return transferId
    }

    fun cancelTransfer(transferId: String) {
        transferJobs[transferId]?.cancel()
        transferJobs.remove(transferId)
        val current = _activeTransfers.value[transferId]
        if (current != null) {
            updateProgress(current.copy(isCancelled = true))
        }
    }

    private fun updateProgress(progress: TransferProgress) {
        val current = _activeTransfers.value.toMutableMap()
        current[progress.transferId] = progress
        _activeTransfers.value = current
    }
}

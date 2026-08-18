package org.novalink.core

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.security.MessageDigest

class ClipboardSyncManager(
    private val context: Context,
    private val onSendClipboardContent: (String) -> Unit
) {
    private val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    private var lastSentHash: String? = null
    private var lastReceivedHash: String? = null

    private val _isSyncEnabled = MutableStateFlow(true)
    val isSyncEnabled: StateFlow<Boolean> = _isSyncEnabled.asStateFlow()

    private val clipListener = ClipboardManager.OnPrimaryClipChangedListener {
        if (!_isSyncEnabled.value) return@OnPrimaryClipChangedListener

        val clip = clipboard.primaryClip ?: return@OnPrimaryClipChangedListener
        if (clip.itemCount > 0) {
            val text = clip.getItemAt(0).text?.toString() ?: return@OnPrimaryClipChangedListener
            if (text.isNotBlank()) {
                val hash = sha256(text)
                // Loop prevention: do not re-broadcast received or identical content
                if (hash == lastReceivedHash || hash == lastSentHash) {
                    return@OnPrimaryClipChangedListener
                }
                lastSentHash = hash
                onSendClipboardContent(text)
            }
        }
    }

    fun startListening() {
        clipboard.addPrimaryClipChangedListener(clipListener)
    }

    fun stopListening() {
        clipboard.removePrimaryClipChangedListener(clipListener)
    }

    fun setSyncEnabled(enabled: Boolean) {
        _isSyncEnabled.value = enabled
    }

    fun applyReceivedClipboard(text: String) {
        if (!_isSyncEnabled.value || text.isBlank()) return
        val hash = sha256(text)
        if (hash == lastSentHash) return // Echo prevention

        lastReceivedHash = hash
        val clip = ClipData.newPlainText("NOVA-Link", text)
        clipboard.setPrimaryClip(clip)
    }

    private fun sha256(input: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(input.toByteArray(Charsets.UTF_8))
        return digest.joinToString("") { "%02x".format(it) }
    }
}

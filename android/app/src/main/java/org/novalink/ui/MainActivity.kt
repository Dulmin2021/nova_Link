package org.novalink.ui

import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import kotlinx.coroutines.launch
import org.novalink.core.ConnectionStatus
import org.novalink.ui.screens.HomeScreen
import org.novalink.ui.screens.PairingDialog
import org.novalink.ui.theme.NOVALinkTheme
import org.novalink.ui.viewmodel.MainViewModel

class MainActivity : ComponentActivity() {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            NOVALinkTheme {
                val devices by viewModel.repository.devices.collectAsState()
                val pairingState by viewModel.pairingDialogState.collectAsState()
                val connectionStatus by viewModel.connectionStatus.collectAsState()
                val userMessage by viewModel.userMessage.collectAsState()
                val snackbarHostState = remember { SnackbarHostState() }
                val scope = rememberCoroutineScope()

                // Show snackbar whenever userMessage is set
                androidx.compose.runtime.LaunchedEffect(userMessage) {
                    userMessage?.let { msg ->
                        snackbarHostState.showSnackbar(msg)
                        viewModel.clearUserMessage()
                    }
                }

                // File picker launcher
                val filePickerLauncher = rememberLauncherForActivityResult(
                    ActivityResultContracts.GetContent()
                ) { uri: Uri? ->
                    uri?.let {
                        val cursor = contentResolver.query(it, null, null, null, null)
                        cursor?.use { c ->
                            val nameIdx = c.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                            val sizeIdx = c.getColumnIndex(OpenableColumns.SIZE)
                            if (c.moveToFirst()) {
                                val name = if (nameIdx >= 0) c.getString(nameIdx) else "file"
                                val size = if (sizeIdx >= 0) c.getLong(sizeIdx) else 0L
                                viewModel.sendFile(it, name, size)
                            }
                        }
                    }
                }

                // Build a human-readable connection status label for the top bar
                val statusLabel: String? = when (val s = connectionStatus) {
                    is ConnectionStatus.Connecting -> "⏳ Connecting to ${s.host}…"
                    is ConnectionStatus.Connected -> "✓ Connected to ${s.host}"
                    is ConnectionStatus.Error -> "⚠ ${s.message}"
                    is ConnectionStatus.Disconnected -> null
                }

                androidx.compose.material3.Scaffold(
                    snackbarHost = { SnackbarHost(snackbarHostState) }
                ) { _ ->
                    HomeScreen(
                        devices = devices,
                        onPairClicked = { viewModel.initiatePairing(it) },
                        onSendFileClicked = { filePickerLauncher.launch("*/*") },
                        onSendText = { text -> viewModel.sendText(text) },
                        onSendUrl = { url -> viewModel.sendUrl(url) },
                        onDirectConnect = { ip, port -> viewModel.connectDirect(ip, port) },
                        connectionStatusLabel = statusLabel
                    )

                    if (pairingState.isVisible) {
                        PairingDialog(
                            deviceName = pairingState.deviceName,
                            sasCode = pairingState.sasCode,
                            onAccept = { viewModel.acceptPairing() },
                            onReject = { viewModel.rejectPairing() }
                        )
                    }
                }
            }
        }
    }
}

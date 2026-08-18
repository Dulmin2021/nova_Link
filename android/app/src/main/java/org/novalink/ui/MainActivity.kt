package org.novalink.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.activity.viewModels
import org.novalink.ui.viewmodel.MainViewModel
import org.novalink.ui.screens.HomeScreen
import org.novalink.ui.theme.NOVALinkTheme

class MainActivity : ComponentActivity() {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            NOVALinkTheme {
                val devices by viewModel.repository.devices.collectAsState()
                val pairingState by viewModel.pairingDialogState.collectAsState()

                HomeScreen(
                    devices = devices,
                    onPairClicked = { viewModel.initiatePairing(it) },
                    onSendFileClicked = { /* Handle file picker */ },
                    onSendTextClicked = { /* Handle text input */ },
                    onDirectConnect = { ip, port -> viewModel.connectDirect(ip, port) }
                )

                if (pairingState.isVisible) {
                    org.novalink.ui.screens.PairingDialog(
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

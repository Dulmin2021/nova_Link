package org.novalink.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import org.novalink.repository.DeviceRepository
import org.novalink.ui.screens.HomeScreen
import org.novalink.ui.theme.NOVALinkTheme

class MainActivity : ComponentActivity() {
    private val deviceRepository = DeviceRepository()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            NOVALinkTheme {
                val devices by deviceRepository.devices.collectAsState()
                HomeScreen(
                    devices = devices,
                    onPairClicked = { /* Handle pairing */ },
                    onSendFileClicked = { /* Handle file picker */ },
                    onSendTextClicked = { /* Handle text input */ }
                )
            }
        }
    }
}

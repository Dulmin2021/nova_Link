package org.novalink.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.novalink.repository.DeviceState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    devices: List<DeviceState>,
    onPairClicked: (DeviceState) -> Unit,
    onSendFileClicked: (DeviceState) -> Unit,
    onSendTextClicked: (DeviceState) -> Unit
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("NOVA-Link") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                )
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
        ) {
            Text(
                text = "Nearby Devices",
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            if (devices.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .weight(1f),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "Searching for nearby Linux devices...",
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            } else {
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.weight(1f)
                ) {
                    items(devices) { device ->
                        DeviceCard(
                            device = device,
                            onPairClicked = { onPairClicked(device) },
                            onSendFileClicked = { onSendFileClicked(device) },
                            onSendTextClicked = { onSendTextClicked(device) }
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun DeviceCard(
    device: DeviceState,
    onPairClicked: () -> Unit,
    onSendFileClicked: () -> Unit,
    onSendTextClicked: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(
                        text = device.info.deviceName,
                        style = MaterialTheme.typography.titleMedium
                    )
                    Text(
                        text = if (device.isConnected) "Connected" else "Available to Pair",
                        style = MaterialTheme.typography.bodySmall,
                        color = if (device.isConnected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline
                    )
                }

                if (!device.isPaired) {
                    Button(onClick = onPairClicked) {
                        Text("Pair")
                    }
                }
            }

            if (device.isConnected) {
                Spacer(modifier = Modifier.height(12.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    OutlinedButton(onClick = onSendFileClicked, modifier = Modifier.weight(1f)) {
                        Text("Send File")
                    }
                    OutlinedButton(onClick = onSendTextClicked, modifier = Modifier.weight(1f)) {
                        Text("Send Text")
                    }
                }
            }
        }
    }
}

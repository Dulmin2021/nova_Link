package org.novalink.ui.screens

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.novalink.repository.DeviceState
import org.novalink.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    devices: List<DeviceState>,
    onPairClicked: (DeviceState) -> Unit,
    onSendFileClicked: (DeviceState) -> Unit,
    onSendText: (String) -> Unit = { _ -> },
    onSendUrl: (String) -> Unit = { _ -> },
    onDirectConnect: (String, Int) -> Unit = { _, _ -> },
    connectionStatusLabel: String? = null
) {
    var showDirectConnectDialog by remember { mutableStateOf(false) }
    var directIp by remember { mutableStateOf("") }
    var searchQuery by remember { mutableStateOf("") }
    var showSendTextDialog by remember { mutableStateOf(false) }
    var showSendUrlDialog by remember { mutableStateOf(false) }
    var sendTextInput by remember { mutableStateOf("") }
    var sendUrlInput by remember { mutableStateOf("") }
    // Which device the quick-action dialog targets
    var quickActionDevice by remember { mutableStateOf<DeviceState?>(null) }

    if (showDirectConnectDialog) {
        AlertDialog(
            onDismissRequest = { showDirectConnectDialog = false },
            title = { Text("Direct IP Connect", fontWeight = FontWeight.Bold) },
            text = {
                Column {
                    Text(
                        text = "Enter your Azure VM or Tailscale IP address:",
                        style = MaterialTheme.typography.bodyMedium,
                        color = NovaSlateMuted,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    OutlinedTextField(
                        value = directIp,
                        onValueChange = { directIp = it },
                        label = { Text("IP Address (e.g. 100.x.x.x)") },
                        singleLine = true,
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        if (directIp.isNotBlank()) {
                            onDirectConnect(directIp.trim(), 42424)
                            showDirectConnectDialog = false
                        }
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = NovaDeepBlue),
                    shape = RoundedCornerShape(8.dp)
                ) {
                    Text("Connect", color = Color.White)
                }
            },
            dismissButton = {
                TextButton(onClick = { showDirectConnectDialog = false }) {
                    Text("Cancel", color = NovaSlateMuted)
                }
            }
        )
    }

    // ---- Send Text Dialog ----
    if (showSendTextDialog) {
        AlertDialog(
            onDismissRequest = { showSendTextDialog = false; sendTextInput = "" },
            title = { Text("Send Text", fontWeight = FontWeight.Bold) },
            text = {
                Column {
                    Text(
                        text = "Enter text to send to your Linux device:",
                        style = MaterialTheme.typography.bodyMedium,
                        color = NovaSlateMuted,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    OutlinedTextField(
                        value = sendTextInput,
                        onValueChange = { sendTextInput = it },
                        label = { Text("Your message") },
                        minLines = 3,
                        maxLines = 6,
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        if (sendTextInput.isNotBlank()) {
                            onSendText(sendTextInput)
                        }
                        showSendTextDialog = false
                        sendTextInput = ""
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = NovaDeepBlue),
                    shape = RoundedCornerShape(8.dp)
                ) { Text("Send", color = Color.White) }
            },
            dismissButton = {
                TextButton(onClick = { showSendTextDialog = false; sendTextInput = "" }) {
                    Text("Cancel", color = NovaSlateMuted)
                }
            }
        )
    }

    // ---- Send URL Dialog ----
    if (showSendUrlDialog) {
        AlertDialog(
            onDismissRequest = { showSendUrlDialog = false; sendUrlInput = "" },
            title = { Text("Share URL", fontWeight = FontWeight.Bold) },
            text = {
                Column {
                    Text(
                        text = "Paste a URL to send to your Linux device:",
                        style = MaterialTheme.typography.bodyMedium,
                        color = NovaSlateMuted,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                    OutlinedTextField(
                        value = sendUrlInput,
                        onValueChange = { sendUrlInput = it },
                        label = { Text("URL (e.g. https://...)") },
                        singleLine = true,
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            },
            confirmButton = {
                Button(
                    onClick = {
                        if (sendUrlInput.isNotBlank()) {
                            onSendUrl(sendUrlInput)
                        }
                        showSendUrlDialog = false
                        sendUrlInput = ""
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = NovaDeepBlue),
                    shape = RoundedCornerShape(8.dp)
                ) { Text("Share", color = Color.White) }
            },
            dismissButton = {
                TextButton(onClick = { showSendUrlDialog = false; sendUrlInput = "" }) {
                    Text("Cancel", color = NovaSlateMuted)
                }
            }
        )
    }

    Scaffold(
        topBar = {
            Column(
                modifier = Modifier
                    .background(Color.White)
                    .fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp, vertical = 14.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Column {
                        Text(
                            text = "Dashboard",
                            fontSize = 22.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                        if (connectionStatusLabel != null) {
                            Text(
                                text = connectionStatusLabel,
                                fontSize = 11.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = if (connectionStatusLabel.startsWith("✓")) NovaMintGreen
                                        else if (connectionStatusLabel.startsWith("⚠")) Color(0xFFD97706)
                                        else NovaSlateMuted,
                                modifier = Modifier.padding(top = 2.dp)
                            )
                        }
                    }

                    Row(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Surface(
                            shape = RoundedCornerShape(20.dp),
                            color = NovaSlateBorder.copy(alpha = 0.6f),
                            modifier = Modifier
                                .clickable { showDirectConnectDialog = true }
                                .padding(horizontal = 12.dp, vertical = 6.dp)
                        ) {
                            Text(
                                text = "🔍 Direct IP",
                                fontSize = 12.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = NovaSlateDark
                            )
                        }

                        Text(
                            text = "🔄",
                            fontSize = 18.sp,
                            modifier = Modifier
                                .clip(CircleShape)
                                .clickable { /* refresh */ }
                                .padding(4.dp)
                        )
                        Text(
                            text = "🔋",
                            fontSize = 18.sp,
                            modifier = Modifier
                                .clip(CircleShape)
                                .clickable { }
                                .padding(4.dp)
                        )
                    }
                }
                Divider(color = NovaSlateBorder, thickness = 1.dp)
            }
        },
        bottomBar = {
            Surface(
                color = NovaCardBg,
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "📶 Local Network: Connected",
                        fontFamily = FontFamily.Monospace,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = NovaMintGreen
                    )
                    Text(
                        text = "NOVA-Link v2.4.1 | Encrypted",
                        fontFamily = FontFamily.Monospace,
                        fontSize = 11.sp,
                        color = NovaSlateMuted
                    )
                }
            }
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .background(NovaBackground)
                .padding(paddingValues)
                .padding(horizontal = 20.dp, vertical = 16.dp)
        ) {
            // ==========================================
            // 1. QUICK ACTIONS SECTION
            // ==========================================
            Text(
                text = "Quick Actions",
                fontSize = 16.sp,
                fontWeight = FontWeight.Bold,
                color = NovaSlateDark,
                modifier = Modifier.padding(bottom = 12.dp)
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                // Card 1: Send File (Deep Blue Primary)
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .clickable {
                            val activeDev = devices.firstOrNull { it.isConnected } ?: devices.firstOrNull()
                            if (activeDev != null) {
                                onSendFileClicked(activeDev)
                            } else {
                                showDirectConnectDialog = true
                            }
                        },
                    shape = RoundedCornerShape(12.dp),
                    color = NovaDeepBlue
                ) {
                    Column(
                        modifier = Modifier.padding(vertical = 18.dp, horizontal = 8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Box(
                            modifier = Modifier
                                .size(40.dp)
                                .clip(CircleShape)
                                .background(Color.White.copy(alpha = 0.2f)),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("📤", fontSize = 18.sp)
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Send File",
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color.White
                        )
                    }
                }

                // Card 2: Send Text (Pale Blue Secondary)
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .clickable {
                            val activeDev = devices.firstOrNull { it.isConnected } ?: devices.firstOrNull()
                            if (activeDev != null) {
                                quickActionDevice = activeDev
                                sendTextInput = ""
                                showSendTextDialog = true
                            } else {
                                showDirectConnectDialog = true
                            }
                        },
                    shape = RoundedCornerShape(12.dp),
                    color = NovaCardBg,
                    border = BorderStroke(1.dp, NovaSlateBorder)
                ) {
                    Column(
                        modifier = Modifier.padding(vertical = 18.dp, horizontal = 8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Box(
                            modifier = Modifier
                                .size(40.dp)
                                .clip(CircleShape)
                                .background(NovaIconBlueCircle),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("💬", fontSize = 18.sp)
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Send Text",
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                    }
                }

                // Card 3: Send URL (Pale Wheat Secondary) — was wrongly calling onSendTextClicked, now fixed
                Surface(
                    modifier = Modifier
                        .weight(1f)
                        .clickable {
                            val activeDev = devices.firstOrNull { it.isConnected } ?: devices.firstOrNull()
                            if (activeDev != null) {
                                quickActionDevice = activeDev
                                sendUrlInput = ""
                                showSendUrlDialog = true
                            } else {
                                showDirectConnectDialog = true
                            }
                        },
                    shape = RoundedCornerShape(12.dp),
                    color = NovaCardBg,
                    border = BorderStroke(1.dp, NovaSlateBorder)
                ) {
                    Column(
                        modifier = Modifier.padding(vertical = 18.dp, horizontal = 8.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Box(
                            modifier = Modifier
                                .size(40.dp)
                                .clip(CircleShape)
                                .background(NovaIconWheatCircle),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("🔗", fontSize = 18.sp)
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "Send URL",
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            // ==========================================
            // 2. NEARBY DEVICES SECTION
            // ==========================================
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 12.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Nearby Devices",
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold,
                    color = NovaSlateDark
                )
                Text(
                    text = "● Scanning active",
                    fontSize = 12.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = NovaMintGreen
                )
            }

            if (devices.isEmpty()) {
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                    shape = RoundedCornerShape(12.dp),
                    color = Color.White,
                    border = BorderStroke(1.dp, NovaSlateBorder)
                ) {
                    Column(
                        modifier = Modifier.padding(32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text("📡", fontSize = 32.sp)
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(
                            text = "No devices detected nearby",
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Tap 'Direct IP' in top right to connect to your Azure VM or PC",
                            fontSize = 12.sp,
                            color = NovaSlateMuted
                        )
                    }
                }
            } else {
                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(devices) { device ->
                        DashboardDeviceCard(
                            device = device,
                            onPairClicked = { onPairClicked(device) },
                            onBrowseClicked = { onSendFileClicked(device) },
                            onMirrorClicked = { /* future: open screen mirror */ }
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun DashboardDeviceCard(
    device: DeviceState,
    onPairClicked: () -> Unit,
    onBrowseClicked: () -> Unit,
    onMirrorClicked: () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(14.dp),
        color = Color.White,
        border = BorderStroke(1.dp, NovaSlateBorder)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Surface(
                        shape = RoundedCornerShape(10.dp),
                        color = NovaCardBg,
                        modifier = Modifier.size(44.dp)
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            Text("💻", fontSize = 22.sp)
                        }
                    }

                    Column {
                        Text(
                            text = device.info.deviceName,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                        Spacer(modifier = Modifier.height(2.dp))
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            if (device.isConnected) {
                                Surface(
                                    shape = RoundedCornerShape(10.dp),
                                    color = NovaMintBadgeBg,
                                    modifier = Modifier.padding(vertical = 1.dp)
                                ) {
                                    Text(
                                        text = "CONNECTED",
                                        fontSize = 10.sp,
                                        fontWeight = FontWeight.ExtraBold,
                                        color = NovaMintBadgeText,
                                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                                    )
                                }
                                Text(
                                    text = "🔋 85%",
                                    fontSize = 11.sp,
                                    color = NovaSlateMuted
                                )
                            } else {
                                Surface(
                                    shape = RoundedCornerShape(10.dp),
                                    color = NovaBadgeOfflineBg
                                ) {
                                    Text(
                                        text = "AVAILABLE",
                                        fontSize = 10.sp,
                                        fontWeight = FontWeight.Bold,
                                        color = NovaBadgeOfflineText,
                                        modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                                    )
                                }
                            }
                        }
                    }
                }

                if (!device.isPaired) {
                    Button(
                        onClick = onPairClicked,
                        colors = ButtonDefaults.buttonColors(containerColor = NovaDeepBlue),
                        shape = RoundedCornerShape(8.dp),
                        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp)
                    ) {
                        Text("Pair", fontSize = 12.sp, fontWeight = FontWeight.Bold, color = Color.White)
                    }
                } else {
                    Text("⋮", fontSize = 20.sp, color = NovaSlateMuted)
                }
            }

            if (device.isConnected) {
                Spacer(modifier = Modifier.height(14.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    OutlinedButton(
                        onClick = onBrowseClicked,
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(8.dp),
                        border = BorderStroke(1.dp, NovaSlateBorder)
                    ) {
                        Text("📁  Browse", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = NovaSlateDark)
                    }
                    OutlinedButton(
                        onClick = onMirrorClicked,
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(8.dp),
                        border = BorderStroke(1.dp, NovaSlateBorder)
                    ) {
                        Text("💻  Mirror", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = NovaSlateDark)
                    }
                }
            }
        }
    }
}

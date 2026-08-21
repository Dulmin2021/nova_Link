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
    connectionStatus: ConnectionStatus = ConnectionStatus.Disconnected
) {
    var showDirectConnectDialog by remember { mutableStateOf(false) }
    var directIp by remember { mutableStateOf("") }
    var searchQuery by remember { mutableStateOf("") }
    var showSendTextDialog by remember { mutableStateOf(false) }
    var showSendUrlDialog by remember { mutableStateOf(false) }
    var sendTextInput by remember { mutableStateOf("") }
    var sendUrlInput by remember { mutableStateOf("") }
    var quickActionDevice by remember { mutableStateOf<DeviceState?>(null) }

    val hasPairedDevice = devices.any { it.isPaired }
    val activeDevice = devices.firstOrNull { it.isConnected } ?: devices.firstOrNull()

    if (showDirectConnectDialog) {
        AlertDialog(
            onDismissRequest = { showDirectConnectDialog = false },
            title = { Text("Direct IP Connect", fontWeight = FontWeight.Bold) },
            text = {
                Column {
                    Text(
                        text = "Enter your Linux PC or Azure/Tailscale IP address:",
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
                            text = "NOVA-Link",
                            fontSize = 22.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                        when (val s = connectionStatus) {
                            is ConnectionStatus.Connecting -> {
                                Text(
                                    text = "⏳ Connecting to ${s.host}…",
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = NovaDeepBlue,
                                    modifier = Modifier.padding(top = 2.dp)
                                )
                            }
                            is ConnectionStatus.Connected -> {
                                Text(
                                    text = "✓ Connected to ${s.host}",
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = NovaMintGreen,
                                    modifier = Modifier.padding(top = 2.dp)
                                )
                            }
                            is ConnectionStatus.Error -> {
                                Text(
                                    text = "⚠ ${s.message}",
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = Color(0xFFD97706),
                                    modifier = Modifier.padding(top = 2.dp)
                                )
                            }
                            is ConnectionStatus.Disconnected -> {
                                Text(
                                    text = "Ready to connect",
                                    fontSize = 11.sp,
                                    color = NovaSlateMuted,
                                    modifier = Modifier.padding(top = 2.dp)
                                )
                            }
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
                    val statusText = when (connectionStatus) {
                        is ConnectionStatus.Connected -> "📶 Local Network: Connected"
                        is ConnectionStatus.Connecting -> "🔄 Establishing Connection..."
                        is ConnectionStatus.Error -> "⚠ Connection Interrupted"
                        is ConnectionStatus.Disconnected -> "📡 Ready to Connect"
                    }
                    val statusColor = when (connectionStatus) {
                        is ConnectionStatus.Connected -> NovaMintGreen
                        is ConnectionStatus.Connecting -> NovaDeepBlue
                        is ConnectionStatus.Error -> Color(0xFFD97706)
                        is ConnectionStatus.Disconnected -> NovaSlateMuted
                    }
                    Text(
                        text = statusText,
                        fontFamily = FontFamily.Monospace,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = statusColor
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
            // STATE 1: Disconnected with no devices -> Show Onboarding CTA Screen
            if (connectionStatus is ConnectionStatus.Disconnected && devices.isEmpty()) {
                Surface(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 12.dp),
                    shape = RoundedCornerShape(16.dp),
                    color = Color.White,
                    border = BorderStroke(1.dp, NovaSlateBorder)
                ) {
                    Column(
                        modifier = Modifier.padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Box(
                            modifier = Modifier
                                .size(64.dp)
                                .clip(CircleShape)
                                .background(NovaCardBg),
                            contentAlignment = Alignment.Center
                        ) {
                            Text("⚡", fontSize = 32.sp)
                        }
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            text = "Connect to your PC",
                            fontSize = 18.sp,
                            fontWeight = FontWeight.Bold,
                            color = NovaSlateDark
                        )
                        Spacer(modifier = Modifier.height(6.dp))
                        Text(
                            text = "Link your Android phone and Linux computer for secure file sharing, clipboard sync, and text sharing.",
                            fontSize = 13.sp,
                            color = NovaSlateMuted,
                            textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                            lineHeight = 18.sp
                        )
                        Spacer(modifier = Modifier.height(20.dp))
                        Button(
                            onClick = { showDirectConnectDialog = true },
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.buttonColors(containerColor = NovaDeepBlue),
                            shape = RoundedCornerShape(10.dp),
                            contentPadding = PaddingValues(vertical = 12.dp)
                        ) {
                            Text("🔗  Enter Tailscale / Direct IP", fontWeight = FontWeight.Bold)
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedButton(
                            onClick = { /* mDNS scan */ },
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(10.dp),
                            border = BorderStroke(1.dp, NovaSlateBorder),
                            contentPadding = PaddingValues(vertical = 12.dp)
                        ) {
                            Text("📡  Scan Local Wi-Fi Network", color = NovaSlateDark, fontWeight = FontWeight.SemiBold)
                        }
                    }
                }
            } else {
                // ==========================================
                // 1. QUICK ACTIONS SECTION (State-Aware)
                // ==========================================
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 12.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "Quick Actions",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                        color = NovaSlateDark
                    )
                    if (!hasPairedDevice && activeDevice != null) {
                        Text(
                            text = "🔒 Pair device to unlock",
                            fontSize = 11.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = NovaSlateMuted
                        )
                    }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    // Card 1: Send File
                    Surface(
                        modifier = Modifier
                            .weight(1f)
                            .clickable {
                                if (activeDevice != null) {
                                    onSendFileClicked(activeDevice)
                                } else {
                                    showDirectConnectDialog = true
                                }
                            },
                        shape = RoundedCornerShape(12.dp),
                        color = if (hasPairedDevice) NovaDeepBlue else NovaCardBg,
                        border = if (hasPairedDevice) null else BorderStroke(1.dp, NovaSlateBorder)
                    ) {
                        Column(
                            modifier = Modifier.padding(vertical = 18.dp, horizontal = 8.dp),
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(40.dp)
                                    .clip(CircleShape)
                                    .background(if (hasPairedDevice) Color.White.copy(alpha = 0.2f) else NovaIconBlueCircle),
                                contentAlignment = Alignment.Center
                            ) {
                                Text("📤", fontSize = 18.sp)
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = "Send File",
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Bold,
                                color = if (hasPairedDevice) Color.White else NovaSlateDark
                            )
                        }
                    }

                    // Card 2: Send Text
                    Surface(
                        modifier = Modifier
                            .weight(1f)
                            .clickable {
                                if (activeDevice != null) {
                                    quickActionDevice = activeDevice
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

                    // Card 3: Send URL
                    Surface(
                        modifier = Modifier
                            .weight(1f)
                            .clickable {
                                if (activeDevice != null) {
                                    quickActionDevice = activeDevice
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
                        text = "Devices",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                        color = NovaSlateDark
                    )
                    Text(
                        text = if (connectionStatus is ConnectionStatus.Connected) "● Connected" else "● Scanning active",
                        fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = NovaMintGreen
                    )
                }

                LazyColumn(
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    items(devices) { device ->
                        DashboardDeviceCard(
                            device = device,
                            onPairClicked = { onPairClicked(device) },
                            onBrowseClicked = { onSendFileClicked(device) },
                            onMirrorClicked = { /* mirror action */ }
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

package org.novalink.core

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.novalink.model.DeviceInfoPayload

class NovaNsdDiscovery(context: Context) {
    private val nsdManager = context.getSystemService(Context.NSD_SERVICE) as NsdManager
    private val tag = "NovaNsdDiscovery"

    private val _discoveredServices = MutableStateFlow<List<NsdServiceInfo>>(emptyList())
    val discoveredServices: StateFlow<List<NsdServiceInfo>> = _discoveredServices.asStateFlow()

    private var discoveryListener: NsdManager.DiscoveryListener? = null
    private var registrationListener: NsdManager.RegistrationListener? = null

    companion object {
        const val SERVICE_TYPE = "_nova-link._tcp."
    }

    fun startDiscovery() {
        if (discoveryListener != null) return

        discoveryListener = object : NsdManager.DiscoveryListener {
            override fun onStartDiscoveryFailed(serviceType: String?, errorCode: Int) {
                Log.e(tag, "Discovery failed to start: $errorCode")
            }

            override fun onStopDiscoveryFailed(serviceType: String?, errorCode: Int) {
                Log.e(tag, "Discovery failed to stop: $errorCode")
            }

            override fun onDiscoveryStarted(serviceType: String?) {
                Log.i(tag, "NOVA-Link service discovery started")
            }

            override fun onDiscoveryStopped(serviceType: String?) {
                Log.i(tag, "NOVA-Link service discovery stopped")
            }

            override fun onServiceFound(serviceInfo: NsdServiceInfo?) {
                serviceInfo?.let { resolveService(it) }
            }

            override fun onServiceLost(serviceInfo: NsdServiceInfo?) {
                serviceInfo?.let { lost ->
                    _discoveredServices.value = _discoveredServices.value.filterNot {
                        it.serviceName == lost.serviceName
                    }
                }
            }
        }

        nsdManager.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, discoveryListener)
    }

    private fun resolveService(serviceInfo: NsdServiceInfo) {
        nsdManager.resolveService(serviceInfo, object : NsdManager.ResolveListener {
            override fun onResolveFailed(serviceInfo: NsdServiceInfo?, errorCode: Int) {
                Log.w(tag, "Resolve failed for ${serviceInfo?.serviceName}: $errorCode")
            }

            override fun onServiceResolved(resolved: NsdServiceInfo?) {
                resolved?.let { info ->
                    val current = _discoveredServices.value.toMutableList()
                    val index = current.indexOfFirst { it.serviceName == info.serviceName }
                    if (index >= 0) {
                        current[index] = info
                    } else {
                        current.add(info)
                    }
                    _discoveredServices.value = current
                }
            }
        })
    }

    fun registerService(deviceName: String, port: Int, attributes: Map<String, String>) {
        val serviceInfo = NsdServiceInfo().apply {
            serviceName = deviceName
            serviceType = SERVICE_TYPE
            this.port = port
            attributes.forEach { (k, v) ->
                setAttribute(k, v)
            }
        }

        registrationListener = object : NsdManager.RegistrationListener {
            override fun onServiceRegistered(NsdServiceInfo: NsdServiceInfo?) {
                Log.i(tag, "Service registered successfully: ${NsdServiceInfo?.serviceName}")
            }

            override fun onRegistrationFailed(serviceInfo: NsdServiceInfo?, errorCode: Int) {
                Log.e(tag, "Service registration failed: $errorCode")
            }

            override fun onServiceUnregistered(arg0: NsdServiceInfo?) {
                Log.i(tag, "Service unregistered successfully")
            }

            override fun onUnregistrationFailed(serviceInfo: NsdServiceInfo?, errorCode: Int) {
                Log.e(tag, "Service unregistration failed: $errorCode")
            }
        }

        nsdManager.registerService(serviceInfo, NsdManager.PROTOCOL_DNS_SD, registrationListener)
    }

    fun stop() {
        discoveryListener?.let {
            try {
                nsdManager.stopServiceDiscovery(it)
            } catch (e: Exception) {
                Log.w(tag, "Error stopping discovery", e)
            }
            discoveryListener = null
        }
        registrationListener?.let {
            try {
                nsdManager.unregisterService(it)
            } catch (e: Exception) {
                Log.w(tag, "Error unregistering service", e)
            }
            registrationListener = null
        }
    }
}

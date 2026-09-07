package io.quicklan.android

import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.net.ConnectivityManager
import android.net.VpnService
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.PersistableBundle
import androidx.annotation.Keep
import org.json.JSONArray
import org.json.JSONObject
import java.net.Inet4Address
import java.net.InetAddress
import java.net.NetworkInterface
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

@Keep
class AndroidPlatform(context: Context) {
    private val app = context.applicationContext
    private val secrets = SecretStore(app)
    private val main = Handler(Looper.getMainLooper())
    private val tunLock = Any()
    private val tunOpen = AtomicBoolean(false)

    @Keep
    fun readSecret(id: String): String? = secrets.read(id)

    @Keep
    fun writeSecret(id: String, value: String): Boolean = secrets.write(id, value)

    @Keep
    fun removeSecret(id: String): Boolean = secrets.remove(id)

    @Keep
    fun establishTun(ip: String, subnet: String): Int {
        val parsed = parseOverlay(ip, subnet) ?: return -1
        if (hasNonDefaultRouteConflict(parsed.network, parsed.prefix)) {
            return -2
        }
        if (VpnService.prepare(app) != null) return -3
        val service = QuickLanVpnService.current() ?: return -1
        if (service.isStopping()) return -1
        synchronized(tunLock) {
            if (tunOpen.get()) return -1
            val fd = service.establishOverlay(parsed.address, parsed.network, parsed.prefix)
            if (fd >= 0) {
                tunOpen.set(true)
            }
            return fd
        }
    }

    @Keep
    fun closeTun() {
        synchronized(tunLock) {
            tunOpen.set(false)
        }
    }

    @Keep
    fun copyText(text: String, sensitive: Boolean): Boolean {
        if (text.length > MAX_CLIP) return false
        return onMain {
            try {
                val clipboard = app.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                val clip = ClipData.newPlainText("QuickLAN", text)
                if (sensitive && Build.VERSION.SDK_INT >= 33) {
                    val extras = PersistableBundle()
                    extras.putBoolean(ClipDescription.EXTRA_IS_SENSITIVE, true)
                    clip.description.extras = extras
                }
                clipboard.setPrimaryClip(clip)
                true
            } catch (_: Exception) {
                false
            }
        }
    }

    @Keep
    fun localEndpoints(): String {
        val choices = ArrayList<JSONObject>()
        val seen = HashSet<String>()
        try {
            val interfaces = NetworkInterface.getNetworkInterfaces() ?: return "[]"
            for (iface in interfaces) {
                if (!iface.isUp || iface.isLoopback) continue
                val name = iface.name ?: continue
                if (name.startsWith("tun") || name.startsWith("vpn") || name.startsWith("dummy")) {
                    continue
                }
                val display = name.filter { !it.isISOControl() }.take(64)
                if (display.isEmpty()) continue
                for (address in iface.inetAddresses) {
                    val ipv4 = address as? Inet4Address ?: continue
                    if (!ipv4.isSiteLocalAddress || ipv4.isLoopbackAddress) continue
                    val host = ipv4.hostAddress ?: continue
                    val endpoint = "tcp://$host:11010"
                    if (!seen.add(endpoint)) continue
                    val row = JSONObject()
                    row.put("interface", display)
                    row.put("endpoint", endpoint)
                    choices.add(row)
                }
            }
        } catch (_: Exception) {
            return "[]"
        }
        choices.sortBy { it.getString("endpoint") }
        val limited = JSONArray()
        for (row in choices.take(32)) {
            limited.put(row)
        }
        return limited.toString()
    }



    private fun hasNonDefaultRouteConflict(network: Inet4Address, prefix: Int): Boolean {
        val requested = ipv4Int(network)
        val cm = app.getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
        val networks = cm.allNetworks
        for (candidate in networks) {
            val properties = cm.getLinkProperties(candidate) ?: continue
            // ConnectivityManager may briefly retain a removed TUN's routes.
            val name = properties.interfaceName
            if (name != null && NetworkInterface.getByName(name) == null) continue
            for (route in properties.routes) {
                val destination = route.destination ?: continue
                if (destination.prefixLength == 0) continue
                val address = destination.address as? Inet4Address ?: continue
                if (ipv4Overlaps(requested, prefix, ipv4Int(address), destination.prefixLength)) {
                    return true
                }
            }
        }
        return false
    }

    private fun <T> onMain(block: () -> T): T {
        if (Looper.myLooper() == Looper.getMainLooper()) {
            return block()
        }
        val latch = CountDownLatch(1)
        val box = arrayOfNulls<Any>(1)
        var failed = false
        main.post {
            try {
                box[0] = block()
            } catch (_: Exception) {
                failed = true
            } finally {
                latch.countDown()
            }
        }
        if (!latch.await(3, TimeUnit.SECONDS) || failed) {
            @Suppress("UNCHECKED_CAST")
            return (false as T)
        }
        @Suppress("UNCHECKED_CAST")
        return box[0] as T
    }

    private data class Overlay(val address: Inet4Address, val network: Inet4Address, val prefix: Int)

    private companion object {
        const val MAX_CLIP = 65_536

        fun parseOverlay(ip: String, subnet: String): Overlay? {
            val slash = subnet.indexOf('/')
            if (slash <= 0 || slash == subnet.lastIndex) return null
            val networkText = subnet.substring(0, slash)
            val prefix = subnet.substring(slash + 1).toIntOrNull() ?: return null
            if (prefix != 24) return null
            val address = parseIpv4(ip) ?: return null
            val network = parseIpv4(networkText) ?: return null
            val mask = (-1 shl (32 - prefix))
            val networkInt = ipv4Int(network)
            if (networkInt != (networkInt and mask)) return null
            val addressInt = ipv4Int(address)
            if (addressInt and mask != networkInt) return null
            return Overlay(address, network, prefix)
        }

        fun parseIpv4(text: String): Inet4Address? {
            val parts = text.split('.')
            if (parts.size != 4) return null
            val bytes = ByteArray(4)
            for (i in 0..3) {
                val part = parts[i]
                if (part.isEmpty() || part.length > 3) return null
                val value = part.toIntOrNull() ?: return null
                if (value !in 0..255 || part != value.toString()) return null
                bytes[i] = value.toByte()
            }
            return InetAddress.getByAddress(bytes) as? Inet4Address
        }

        fun ipv4Int(address: Inet4Address): Int {
            val bytes = address.address
            return (bytes[0].toInt() and 0xff shl 24) or
                (bytes[1].toInt() and 0xff shl 16) or
                (bytes[2].toInt() and 0xff shl 8) or
                (bytes[3].toInt() and 0xff)
        }

        fun ipv4Overlaps(a: Int, aPrefix: Int, b: Int, bPrefix: Int): Boolean {
            val bits = minOf(aPrefix, bPrefix)
            if (bits <= 0) return false
            val mask = -1 shl (32 - bits)
            return (a and mask) == (b and mask)
        }
    }
}

package io.quicklan.android

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.VpnService
import android.os.Build
import android.os.SystemClock
import android.system.OsConstants
import androidx.core.app.NotificationCompat
import java.net.Inet4Address
import java.util.concurrent.atomic.AtomicBoolean

class QuickLanVpnService : VpnService() {
    private val stopped = AtomicBoolean(false)

    override fun onCreate() {
        super.onCreate()
        createChannel()
        publish(this)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForegroundNotification()
        when (intent?.action) {
            ACTION_DISCONNECT -> {
                QuickLanApplication.instance.executor.disconnectFromSystem()
            }
            ACTION_STOP -> {
                requestStop()
            }
        }
        return START_NOT_STICKY
    }

    override fun onRevoke() {
        super.onRevoke()
        QuickLanApplication.instance.platform.closeTun()
        QuickLanApplication.instance.executor.disconnectFromSystem()
    }

    override fun onDestroy() {
        publish(null)
        if (!stopped.get() && !QuickLanApplication.instance.executor.quitting) {
            QuickLanApplication.instance.executor.disconnectFromSystem()
        }
        super.onDestroy()
    }

    fun establishOverlay(address: Inet4Address, network: Inet4Address, prefix: Int): Int {
        return try {
            val builder =
                Builder()
                    .setSession("QuickLAN")
                    .setMtu(MTU)
                    .setBlocking(true)
                    .addAddress(address, prefix)
                    .addRoute(network, prefix)
                    .allowFamily(OsConstants.AF_INET6)
            if (Build.VERSION.SDK_INT >= 29) {
                builder.setMetered(false)
            }
            val tun = builder.establish() ?: return -1
            tun.detachFd()
        } catch (_: Exception) {
            -1
        }
    }


    fun isStopping(): Boolean = stopped.get()

    fun requestStop() {
        if (!stopped.compareAndSet(false, true)) return
        try {
            stopForeground(STOP_FOREGROUND_REMOVE)
        } catch (_: Exception) {
        }
        stopSelf()
    }

    private fun startForegroundNotification() {
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= 34) {
            startForeground(
                NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE,
            )
        } else {
            startForeground(NOTIFICATION_ID, notification)
        }
    }

    private fun buildNotification(): Notification {
        val open =
            PendingIntent.getActivity(
                this,
                0,
                Intent(this, MainActivity::class.java)
                    .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT),
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
            )
        val disconnect =
            PendingIntent.getService(
                this,
                1,
                Intent(this, QuickLanVpnService::class.java).setAction(ACTION_DISCONNECT),
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
            )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(getString(R.string.notification_title))
            .setContentText(getString(R.string.notification_text))
            .setContentIntent(open)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setCategory(NotificationCompat.CATEGORY_SERVICE)
            .addAction(0, getString(R.string.notification_disconnect), disconnect)
            .build()
    }

    private fun createChannel() {
        val manager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        val channel =
            NotificationChannel(
                CHANNEL_ID,
                getString(R.string.notification_channel),
                NotificationManager.IMPORTANCE_LOW,
            )
        channel.setShowBadge(false)
        channel.description = getString(R.string.notification_text)
        manager.createNotificationChannel(channel)
    }

    companion object {
        const val ACTION_START = "io.quicklan.android.VPN_START"
        const val ACTION_DISCONNECT = "io.quicklan.android.VPN_DISCONNECT"
        const val ACTION_STOP = "io.quicklan.android.VPN_STOP"
        private const val CHANNEL_ID = "quicklan.vpn"
        private const val NOTIFICATION_ID = 1
        private const val MTU = 1360
        private val lock = Object()

        @Volatile
        private var running: QuickLanVpnService? = null

        fun current(): QuickLanVpnService? = running



        fun awaitRunning(timeoutMs: Long): QuickLanVpnService? {
            val deadline = SystemClock.elapsedRealtime() + timeoutMs
            synchronized(lock) {
                while (running == null) {
                    val remaining = deadline - SystemClock.elapsedRealtime()
                    if (remaining <= 0L) return null
                    lock.wait(remaining)
                }
                return running
            }
        }

        fun awaitStopped(timeoutMs: Long): Boolean {
            val deadline = SystemClock.elapsedRealtime() + timeoutMs
            synchronized(lock) {
                while (running != null) {
                    val remaining = deadline - SystemClock.elapsedRealtime()
                    if (remaining <= 0L) return false
                    lock.wait(remaining)
                }
                return true
            }
        }

        private fun publish(service: QuickLanVpnService?) {
            synchronized(lock) {
                running = service
                lock.notifyAll()
            }
        }
    }
}

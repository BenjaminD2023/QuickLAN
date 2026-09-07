package io.quicklan.android

import android.app.Application
import android.content.Intent
import android.os.Handler
import android.os.Looper
import androidx.core.content.ContextCompat

class QuickLanApplication : Application() {
    lateinit var platform: AndroidPlatform
        private set
    lateinit var executor: CommandExecutor
        private set

    val mainHandler = Handler(Looper.getMainLooper())

    @Volatile
    var activity: MainActivity? = null

    override fun onCreate() {
        super.onCreate()
        instance = this
        platform = AndroidPlatform(this)
        executor = CommandExecutor(this)
        executor.start()
    }

    fun startVpnService(): Boolean {
        val existing = QuickLanVpnService.current()
        if (existing != null && !existing.isStopping()) return true
        if (existing != null && !QuickLanVpnService.awaitStopped(5_000)) return false
        return try {
            val intent = Intent(this, QuickLanVpnService::class.java)
                .setAction(QuickLanVpnService.ACTION_START)
            ContextCompat.startForegroundService(this, intent)
            QuickLanVpnService.awaitRunning(8_000) != null
        } catch (_: Exception) {
            false
        }
    }

    fun stopVpnService() {
        val running = QuickLanVpnService.current()
        if (running != null) {
            running.requestStop()
            return
        }
        stopService(Intent(this, QuickLanVpnService::class.java))
    }

    companion object {
        lateinit var instance: QuickLanApplication
            private set
    }
}

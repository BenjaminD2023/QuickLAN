package io.quicklan.android

import android.net.VpnService
import org.json.JSONObject
import java.util.concurrent.Executors
import java.util.concurrent.ThreadFactory
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

class CommandExecutor(private val app: QuickLanApplication) {
    private val worker =
        Executors.newSingleThreadScheduledExecutor(
            ThreadFactory { runnable ->
                Thread(runnable, "quicklan-cmd").apply { isDaemon = false }
            },
        )

    private val started = AtomicBoolean(false)
    private val nativeReady = AtomicBoolean(false)
    private val shuttingDown = AtomicBoolean(false)
    private val pendingFinish = AtomicBoolean(false)

    @Volatile
    private var initError: String = "helper_unavailable"

    val quitting: Boolean
        get() = shuttingDown.get()

    fun start() {
        if (!started.compareAndSet(false, true)) return
        worker.execute { ensureNative() }
        worker.scheduleWithFixedDelay({
            if (nativeReady.get() && QuickLanVpnService.current() != null) {
                val state = runNative("get_state", JSONObject())
                val phase = state.optJSONObject("value")?.optJSONObject("connection")?.optString("phase")
                if (!state.optBoolean("ok") || phase == "failed" || phase == "disconnected") {
                    app.stopVpnService()
                }
            }
        }, 2, 2, TimeUnit.SECONDS)
    }

    fun submitJs(json: String, deliver: (JSONObject) -> Unit) {
        worker.execute {
            handleMessage(json) { payload ->
                val finish = pendingFinish.getAndSet(false)
                app.mainHandler.post {
                    deliver(payload)
                    if (finish) {
                        app.activity?.finishAffinity()
                    }
                }
            }
        }
    }

    fun disconnectFromSystem() {
        worker.execute {
            app.activity?.cancelVpnConsent()
            if (shuttingDown.get()) return@execute
            if (nativeReady.get()) {
                runNative("disconnect_network", JSONObject())
            }
            app.stopVpnService()
        }
    }

    private fun handleMessage(json: String, respond: (JSONObject) -> Unit) {
        if (json.length > MAX_MESSAGE) {
            return respond(receive("", ok = false, error = "unauthorized"))
        }
        val message =
            try {
                JSONObject(json)
            } catch (_: Exception) {
                return respond(receive("", ok = false, error = "unauthorized"))
            }
        val id = message.optString("id")
        val command = message.optString("command")
        if (id.isEmpty() || id.length > 128 || command.isEmpty() || command.length > 64) {
            return respond(receive(id, ok = false, error = "unauthorized"))
        }
        if (!ID.matches(id) || !COMMAND.matches(command)) {
            return respond(receive(id, ok = false, error = "unauthorized"))
        }
        val args = message.optJSONObject("args") ?: JSONObject()
        if (command == "connect_network" && !shuttingDown.get()) {
            val consent = VpnService.prepare(app)
            if (consent != null) {
                val activity = app.activity
                    ?: return respond(receive(id, ok = false, error = "permission_denied"))
                // User consent must not block state polling or disconnect commands.
                activity.requestVpnConsent(consent) { approved ->
                    worker.execute {
                        val result = if (approved) dispatch(command, args)
                            else envelope(ok = false, error = "permission_denied")
                        respond(receiveFromEnvelope(id, result))
                    }
                }
                return
            }
        }
        respond(receiveFromEnvelope(id, dispatch(command, args)))
    }

    private fun dispatch(command: String, args: JSONObject): JSONObject {
        if (shuttingDown.get() && command != "quit_app") {
            return envelope(ok = false, error = "invalid_transition")
        }
        return when (command) {
            "connect_network" -> connect(args)
            "disconnect_network" -> disconnect(args)
            "quit_app" -> quit(args)
            else -> {
                if (!ensureNative()) {
                    return envelope(ok = false, error = initError)
                }
                runNative(command, args)
            }
        }
    }

    private fun connect(args: JSONObject): JSONObject {
        if (!ensureNative()) {
            return envelope(ok = false, error = initError)
        }
        if (VpnService.prepare(app) != null) {
            return envelope(ok = false, error = "permission_denied")
        }
        val alreadyRunning = QuickLanVpnService.current() != null
        if (!app.startVpnService()) {
            return envelope(ok = false, error = "core_failed")
        }
        val result = runNative("connect_network", args)
        if (!alreadyRunning && (!result.optBoolean("ok", false) ||
                result.optJSONObject("value")?.optString("phase") == "failed")) {
            app.stopVpnService()
        }
        return result
    }

    private fun disconnect(args: JSONObject): JSONObject {
        app.activity?.cancelVpnConsent()
        val result =
            if (ensureNative()) {
                runNative("disconnect_network", args)
            } else {
                envelope(ok = false, error = initError)
            }
        app.stopVpnService()
        return result
    }

    private fun quit(args: JSONObject): JSONObject {
        app.activity?.cancelVpnConsent()
        shuttingDown.set(true)
        return try {
            val result =
                if (ensureNative()) {
                    runNative("quit_app", args)
                } else {
                    envelope(ok = true, value = JSONObject.NULL)
                }
            app.stopVpnService()
            pendingFinish.set(true)
            result
        } finally {
            shuttingDown.set(false)
        }
    }

    private fun ensureNative(): Boolean {
        if (nativeReady.get()) return true
        if (shuttingDown.get()) return false
        return try {
            val directory = app.filesDir.absolutePath
            val raw = NativeBridge.initialize(directory, app.platform)
            val parsed = parseEnvelope(raw)
            if (parsed.optBoolean("ok", false)) {
                nativeReady.set(true)
                true
            } else {
                initError = parsed.optString("error").ifEmpty { "helper_unavailable" }
                false
            }
        } catch (_: UnsatisfiedLinkError) {
            initError = "helper_unavailable"
            false
        } catch (_: Exception) {
            initError = "helper_unavailable"
            false
        }
    }

    private fun runNative(command: String, args: JSONObject): JSONObject {
        return try {
            parseEnvelope(NativeBridge.invoke(command, args.toString()))
        } catch (_: Exception) {
            envelope(ok = false, error = "core_failed")
        }
    }

    private companion object {
        const val MAX_MESSAGE = 65_536
        val ID = Regex("^[A-Za-z0-9._:-]{1,128}$")
        val COMMAND = Regex("^[a-z_]{1,64}$")

        fun envelope(ok: Boolean, error: String? = null, value: Any? = JSONObject.NULL): JSONObject {
            val body = JSONObject()
            body.put("ok", ok)
            if (ok) {
                body.put("value", value ?: JSONObject.NULL)
            } else {
                body.put("error", error ?: "core_failed")
            }
            return body
        }

        fun parseEnvelope(raw: String): JSONObject {
            return try {
                val parsed = JSONObject(raw)
                if (!parsed.has("ok")) {
                    envelope(ok = false, error = "core_failed")
                } else {
                    parsed
                }
            } catch (_: Exception) {
                envelope(ok = false, error = "core_failed")
            }
        }

        fun receiveFromEnvelope(id: String, envelope: JSONObject): JSONObject {
            val ok = envelope.optBoolean("ok", false)
            return if (ok) {
                val value =
                    if (envelope.has("value")) envelope.get("value") else JSONObject.NULL
                receive(id, ok = true, value = value, error = null)
            } else {
                receive(
                    id,
                    ok = false,
                    error = envelope.optString("error").ifEmpty { "core_failed" },
                )
            }
        }

        fun receive(
            id: String,
            ok: Boolean,
            value: Any? = JSONObject.NULL,
            error: String? = null,
        ): JSONObject {
            val payload = JSONObject()
            payload.put("id", id)
            payload.put("ok", ok)
            if (ok) {
                payload.put("value", value ?: JSONObject.NULL)
                payload.put("error", JSONObject.NULL)
            } else {
                payload.put("value", JSONObject.NULL)
                payload.put("error", error ?: "core_failed")
            }
            return payload
        }
    }
}

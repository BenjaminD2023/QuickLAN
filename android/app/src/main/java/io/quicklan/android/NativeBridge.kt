package io.quicklan.android

import androidx.annotation.Keep

@Keep
object NativeBridge {
    init {
        System.loadLibrary("quicklan_android")
    }

    external fun initialize(directory: String, platform: AndroidPlatform): String

    external fun invoke(command: String, argsJson: String): String

    external fun shutdown()
}

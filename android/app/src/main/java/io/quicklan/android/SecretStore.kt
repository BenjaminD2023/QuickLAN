package io.quicklan.android

import android.content.Context
import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.AtomicFile
import java.io.File
import java.security.KeyStore
import javax.crypto.AEADBadTagException
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

internal class SecretStore(context: Context) {
    private val lock = Any()
    private val directory = File(context.noBackupFilesDir, "secrets")

    fun read(id: String): String? = synchronized(lock) {
        val file = fileFor(id) ?: return null
        if (!file.exists() || file.length() > MAX_BLOB) return null
        val key = getKey(create = false) ?: return null
        return try {
            val packed = AtomicFile(file).readFully()
            if (packed.size < HEADER + IV_LEN + TAG_LEN || packed.size > MAX_BLOB) {
                return null
            }
            if (!packed.copyOfRange(0, MAGIC.size).contentEquals(MAGIC)) {
                return null
            }
            val iv = packed.copyOfRange(HEADER, HEADER + IV_LEN)
            val ciphertext = packed.copyOfRange(HEADER + IV_LEN, packed.size)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(TAG_BITS, iv))
            cipher.updateAAD(id.toByteArray(Charsets.UTF_8))
            String(cipher.doFinal(ciphertext), Charsets.UTF_8)
        } catch (_: AEADBadTagException) {
            null
        } catch (_: Exception) {
            null
        }
    }

    fun write(id: String, value: String): Boolean = synchronized(lock) {
        if (value.isEmpty() || value.length > MAX_SECRET_CHARS) return false
        val file = fileFor(id) ?: return false
        val key = getKey(create = true) ?: return false
        val atomic = AtomicFile(file)
        var stream: java.io.FileOutputStream? = null
        return try {
            if (!directory.exists() && !directory.mkdirs()) return false
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, key)
            cipher.updateAAD(id.toByteArray(Charsets.UTF_8))
            val iv = cipher.iv
            if (iv.size != IV_LEN) return false
            val ciphertext = cipher.doFinal(value.toByteArray(Charsets.UTF_8))
            if (ciphertext.size > MAX_BLOB) return false
            val packed = ByteArray(HEADER + iv.size + ciphertext.size)
            MAGIC.copyInto(packed)
            iv.copyInto(packed, HEADER)
            ciphertext.copyInto(packed, HEADER + iv.size)
            stream = atomic.startWrite()
            stream.write(packed)
            atomic.finishWrite(stream)
            stream = null
            true
        } catch (_: Exception) {
            if (stream != null) {
                try {
                    atomic.failWrite(stream)
                } catch (_: Exception) {
                }
            }
            false
        }
    }

    fun remove(id: String): Boolean = synchronized(lock) {
        val file = fileFor(id) ?: return false
        if (!file.exists()) return true
        return try {
            AtomicFile(file).delete()
            !file.exists()
        } catch (_: Exception) {
            !file.exists()
        }
    }

    private fun fileFor(id: String): File? {
        if (id.isEmpty() || id.length > MAX_ID) return null
        if (!id.all { it.isLetterOrDigit() || it == '.' || it == '_' || it == '-' }) return null
        return File(directory, "$id.box")
    }

    private fun getKey(create: Boolean): SecretKey? {
        return try {
            val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE)
            keyStore.load(null)
            val existing = keyStore.getEntry(ALIAS, null) as? KeyStore.SecretKeyEntry
            if (existing != null) {
                return existing.secretKey
            }
            if (!create) return null
            val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
            val builder =
                KeyGenParameterSpec.Builder(
                    ALIAS,
                    KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
                )
                    .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                    .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                    .setKeySize(256)
                    .setRandomizedEncryptionRequired(true)
            if (Build.VERSION.SDK_INT >= 28) {
                builder.setUnlockedDeviceRequired(false)
            }
            generator.init(builder.build())
            generator.generateKey()
        } catch (_: Exception) {
            null
        }
    }

    private companion object {
        const val ANDROID_KEYSTORE = "AndroidKeyStore"
        const val ALIAS = "io.quicklan.android.secrets"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val IV_LEN = 12
        const val TAG_BITS = 128
        const val TAG_LEN = 16
        const val VERSION: Byte = 1
        val MAGIC = byteArrayOf('Q'.code.toByte(), 'L'.code.toByte(), 'S'.code.toByte(), VERSION)
        const val HEADER = 4
        const val MAX_ID = 128
        const val MAX_SECRET_CHARS = 8_192
        const val MAX_BLOB = 32_768
    }
}

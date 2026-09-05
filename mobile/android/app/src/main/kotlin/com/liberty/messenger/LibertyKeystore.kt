package com.liberty.messenger

import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

object LibertyKeystore {
    private const val ANDROID_KEYSTORE = "AndroidKeyStore"
    private const val ALIAS = "liberty-vault-wrap"
    private const val TRANSFORMATION = "AES/GCM/NoPadding"
    private const val GCM_TAG_BITS = 128

    private fun keyStore(): KeyStore =
        KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }

    fun ensureKey(useStrongBox: Boolean = true): Boolean {
        val ks = keyStore()
        if (ks.containsAlias(ALIAS)) return true
        val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        val builder = KeyGenParameterSpec.Builder(
            ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setUserAuthenticationRequired(false)
        if (useStrongBox && Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            try { builder.setIsStrongBoxBacked(true) } catch (_: Exception) {}
        }
        return try {
            keyGenerator.init(builder.build())
            keyGenerator.generateKey()
            true
        } catch (_: Exception) {
            if (useStrongBox) ensureKey(false) else false
        }
    }

    private fun secretKey(): SecretKey {
        val ks = keyStore()
        return (ks.getEntry(ALIAS, null) as KeyStore.SecretKeyEntry).secretKey
    }

    fun wrap(data: ByteArray): String {
        ensureKey()
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, secretKey())
        val iv = cipher.iv
        val ct = cipher.doFinal(data)
        val out = ByteArray(iv.size + ct.size)
        System.arraycopy(iv, 0, out, 0, iv.size)
        System.arraycopy(ct, 0, out, iv.size, ct.size)
        return Base64.encodeToString(out, Base64.NO_WRAP)
    }

    fun unwrap(wrappedB64: String): ByteArray {
        val all = Base64.decode(wrappedB64, Base64.NO_WRAP)
        val iv = all.copyOfRange(0, 12)
        val ct = all.copyOfRange(12, all.size)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.DECRYPT_MODE, secretKey(), GCMParameterSpec(GCM_TAG_BITS, iv))
        return cipher.doFinal(ct)
    }

    fun deleteKey() {
        try { keyStore().deleteEntry(ALIAS) } catch (_: Exception) {}
    }

    fun isHardwareBacked(): Boolean {
        return try {
            ensureKey()
            true
        } catch (_: Exception) { false }
    }
}

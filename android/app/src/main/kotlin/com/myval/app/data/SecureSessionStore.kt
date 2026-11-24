package com.myval.app.data

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import org.json.JSONArray
import org.json.JSONObject
import uniffi.myval_ffi.SessionSummary
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

data class StoredAccountRecord(
    val puuid: String,
    val riotId: String,
    val region: String,
    val shard: String,
    val sessionJson: String,
    val ssoCookies: String? = null,
    val rankTierName: String? = null,
    val rankColorHex: String? = null,
    val rankIconUrl: String? = null,
    val expiresAtEpochSecs: Long = 0L,
    val addedAtEpochMillis: Long = System.currentTimeMillis(),
)

data class AccountVault(
    val activePuuid: String? = null,
    val accounts: List<StoredAccountRecord> = emptyList(),
)

/**
 * Encrypts and persists multiple Riot accounts + optional SSO cookie jars inside
 * Private SharedPreferences using a hardware-backed AES-256-GCM key in AndroidKeyStore.
 */
class SecureSessionStore(context: Context) {
    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    @Synchronized
    fun loadVault(): AccountVault {
        val rawEncryptedVault = prefs.getString(KEY_VAULT, null)
        if (!rawEncryptedVault.isNullOrBlank()) {
            val decrypted = decryptString(rawEncryptedVault)
            if (decrypted != null) {
                parseVaultJson(decrypted)?.let { return it }
            }
        }

        // Seamless one-time migration from single-account KEY_SESSION storage.
        val legacyPayload = prefs.getString(KEY_SESSION, null)
        val legacyJson = legacyPayload?.let(::decryptString)
        val migratedRecord = legacyJson?.let(::recordFromLegacySessionJson)
        if (migratedRecord != null) {
            val vault = AccountVault(
                activePuuid = migratedRecord.puuid,
                accounts = listOf(migratedRecord),
            )
            saveVault(vault)
            prefs.edit().remove(KEY_SESSION).apply()
            return vault
        }

        // Discard undecryptable vault/legacy blobs so corrupted Keystore states never trap users
        if (!rawEncryptedVault.isNullOrBlank() || !legacyPayload.isNullOrBlank()) {
            prefs.edit().remove(KEY_VAULT).remove(KEY_SESSION).apply()
        }
        return AccountVault()
    }

    @Synchronized
    fun upsertAccount(
        summary: SessionSummary,
        sessionJson: String,
        ssoCookies: String? = null,
        setActive: Boolean = true,
    ): AccountVault {
        val current = loadVault()
        val existing = current.accounts.firstOrNull { it.puuid.equals(summary.puuid, ignoreCase = true) }
        val resolvedCookies = ssoCookies?.takeIf { it.isNotBlank() } ?: existing?.ssoCookies
        val updatedEntry = StoredAccountRecord(
            puuid = summary.puuid,
            riotId = summary.riotId,
            region = summary.region,
            shard = summary.shard,
            sessionJson = sessionJson,
            ssoCookies = resolvedCookies,
            rankTierName = existing?.rankTierName,
            rankColorHex = existing?.rankColorHex,
            rankIconUrl = existing?.rankIconUrl,
            expiresAtEpochSecs = summary.expiresAtEpochSecs,
            addedAtEpochMillis = existing?.addedAtEpochMillis ?: System.currentTimeMillis(),
        )
        val filtered = current.accounts.filterNot { it.puuid.equals(summary.puuid, ignoreCase = true) }
        val newList = if (existing != null) {
            current.accounts.map {
                if (it.puuid.equals(summary.puuid, ignoreCase = true)) updatedEntry else it
            }
        } else {
            filtered + updatedEntry
        }
        val nextActive = if (setActive) summary.puuid else (current.activePuuid ?: summary.puuid)
        return AccountVault(activePuuid = nextActive, accounts = newList).also(::saveVault)
    }

    @Synchronized
    fun updateAccountRank(
        puuid: String,
        rankTierName: String,
        rankColorHex: String,
        rankIconUrl: String?,
    ): AccountVault {
        val current = loadVault()
        val nextList = current.accounts.map { acc ->
            if (acc.puuid.equals(puuid, ignoreCase = true)) {
                acc.copy(
                    rankTierName = rankTierName,
                    rankColorHex = rankColorHex,
                    rankIconUrl = rankIconUrl,
                )
            } else {
                acc
            }
        }
        return current.copy(accounts = nextList).also(::saveVault)
    }

    @Synchronized
    fun attachCookiesIfBlank(puuid: String, ssoCookies: String?): AccountVault {
        val clean = ssoCookies?.takeIf { it.isNotBlank() } ?: return loadVault()
        val current = loadVault()
        val nextList = current.accounts.map { acc ->
            if (acc.puuid.equals(puuid, ignoreCase = true) && acc.ssoCookies.isNullOrBlank()) {
                acc.copy(ssoCookies = clean)
            } else {
                acc
            }
        }
        return current.copy(accounts = nextList).also(::saveVault)
    }

    @Synchronized
    fun setActivePuuid(puuid: String?): AccountVault {
        val current = loadVault()
        val nextActive = puuid?.takeIf { target ->
            current.accounts.any { it.puuid.equals(target, ignoreCase = true) }
        }
        return current.copy(activePuuid = nextActive).also(::saveVault)
    }

    @Synchronized
    fun removeAccount(puuid: String): AccountVault {
        val current = loadVault()
        val remaining = current.accounts.filterNot { it.puuid.equals(puuid, ignoreCase = true) }
        val nextActive = when {
            remaining.isEmpty() -> null
            current.activePuuid.equals(puuid, ignoreCase = true) -> remaining.first().puuid
            else -> current.activePuuid
        }
        return AccountVault(activePuuid = nextActive, accounts = remaining).also(::saveVault)
    }

    @Synchronized
    fun clearAll() {
        prefs.edit().remove(KEY_VAULT).remove(KEY_SESSION).apply()
    }

    private fun saveVault(vault: AccountVault) {
        val root = JSONObject().apply {
            put("activePuuid", vault.activePuuid ?: JSONObject.NULL)
            val arr = JSONArray()
            vault.accounts.forEach { acc ->
                arr.put(
                    JSONObject().apply {
                        put("puuid", acc.puuid)
                        put("riotId", acc.riotId)
                        put("region", acc.region)
                        put("shard", acc.shard)
                        put("sessionJson", acc.sessionJson)
                        put("ssoCookies", acc.ssoCookies ?: JSONObject.NULL)
                        put("rankTierName", acc.rankTierName ?: JSONObject.NULL)
                        put("rankColorHex", acc.rankColorHex ?: JSONObject.NULL)
                        put("rankIconUrl", acc.rankIconUrl ?: JSONObject.NULL)
                        put("expiresAtEpochSecs", acc.expiresAtEpochSecs)
                        put("addedAtEpochMillis", acc.addedAtEpochMillis)
                    },
                )
            }
            put("accounts", arr)
        }
        val encrypted = encryptString(root.toString())
        prefs.edit().putString(KEY_VAULT, encrypted).remove(KEY_SESSION).apply()
    }

    private fun parseVaultJson(rawJson: String): AccountVault? = runCatching {
        val root = JSONObject(rawJson)
        val active = root.optString("activePuuid", "").takeIf { it.isNotBlank() && !root.isNull("activePuuid") }
        val arr = root.optJSONArray("accounts") ?: JSONArray()
        val list = buildList {
            for (i in 0 until arr.length()) {
                val o = arr.optJSONObject(i) ?: continue
                val puuid = o.optString("puuid", "")
                val sessionJson = o.optString("sessionJson", "")
                if (puuid.isBlank() || sessionJson.isBlank()) continue
                add(
                    StoredAccountRecord(
                        puuid = puuid,
                        riotId = o.optString("riotId", puuid.take(8)),
                        region = o.optString("region", "na"),
                        shard = o.optString("shard", "na"),
                        sessionJson = sessionJson,
                        ssoCookies = if (o.isNull("ssoCookies")) null else o.optString("ssoCookies").takeIf { it.isNotBlank() },
                        rankTierName = if (o.isNull("rankTierName")) null else o.optString("rankTierName").takeIf { it.isNotBlank() },
                        rankColorHex = if (o.isNull("rankColorHex")) null else o.optString("rankColorHex").takeIf { it.isNotBlank() },
                        rankIconUrl = if (o.isNull("rankIconUrl")) null else o.optString("rankIconUrl").takeIf { it.isNotBlank() },
                        expiresAtEpochSecs = o.optLong("expiresAtEpochSecs", 0L),
                        addedAtEpochMillis = o.optLong("addedAtEpochMillis", System.currentTimeMillis()),
                    ),
                )
            }
        }
        val resolvedActive = active?.takeIf { target -> list.any { it.puuid.equals(target, ignoreCase = true) } }
            ?: list.firstOrNull()?.puuid
        AccountVault(activePuuid = resolvedActive, accounts = list)
    }.getOrNull()

    private fun recordFromLegacySessionJson(sessionJson: String): StoredAccountRecord? = runCatching {
        val obj = JSONObject(sessionJson)
        val puuid = obj.optString("puuid", "")
        if (puuid.isBlank()) return@runCatching null
        val gameName = obj.optString("game_name", "")
        val tagLine = obj.optString("tag_line", "")
        val riotId = when {
            gameName.isNotBlank() && tagLine.isNotBlank() -> "$gameName#$tagLine"
            gameName.isNotBlank() -> gameName
            else -> puuid.take(8)
        }
        StoredAccountRecord(
            puuid = puuid,
            riotId = riotId,
            region = obj.optString("region", "na"),
            shard = obj.optString("shard", "na"),
            sessionJson = sessionJson,
            expiresAtEpochSecs = obj.optLong("expires_at_epoch_secs", 0L),
        )
    }.getOrNull()

    private fun encryptString(plaintext: String): String {
        return try {
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, getOrCreateSecretKey())
            val iv = cipher.iv
            val ciphertext = cipher.doFinal(plaintext.toByteArray(Charsets.UTF_8))
            "${Base64.encodeToString(iv, Base64.NO_WRAP)}:${Base64.encodeToString(ciphertext, Base64.NO_WRAP)}"
        } catch (_: Exception) {
            // Reset corrupted KeyStore entry and retry once cleanly
            resetSecretKey()
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.ENCRYPT_MODE, getOrCreateSecretKey())
            val iv = cipher.iv
            val ciphertext = cipher.doFinal(plaintext.toByteArray(Charsets.UTF_8))
            "${Base64.encodeToString(iv, Base64.NO_WRAP)}:${Base64.encodeToString(ciphertext, Base64.NO_WRAP)}"
        }
    }

    private fun decryptString(payload: String): String? {
        val parts = payload.split(':', limit = 2)
        if (parts.size != 2) return null
        return runCatching {
            val iv = Base64.decode(parts[0], Base64.NO_WRAP)
            val ciphertext = Base64.decode(parts[1], Base64.NO_WRAP)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, getOrCreateSecretKey(), GCMParameterSpec(128, iv))
            String(cipher.doFinal(ciphertext), Charsets.UTF_8)
        }.getOrNull()
    }

    private fun resetSecretKey() {
        runCatching {
            val ks = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
            if (ks.containsAlias(KEY_ALIAS)) {
                ks.deleteEntry(KEY_ALIAS)
            }
        }
    }

    private fun getOrCreateSecretKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        val existing = runCatching { keyStore.getKey(KEY_ALIAS, null) as? SecretKey }
            .onFailure { resetSecretKey() }
            .getOrNull()
        if (existing != null) return existing

        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        val spec = KeyGenParameterSpec.Builder(
            KEY_ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .build()
        generator.init(spec)
        return generator.generateKey()
    }

    private companion object {
        const val ANDROID_KEYSTORE = "AndroidKeyStore"
        const val KEY_ALIAS = "myval_session_aes_key"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val PREFS_NAME = "myval_secure_session"
        const val KEY_SESSION = "encrypted_riot_session"
        const val KEY_VAULT = "encrypted_riot_accounts_vault"
    }
}

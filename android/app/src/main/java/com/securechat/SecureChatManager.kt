package com.securechat

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.securechat.crypto.CryptoManager
import com.securechat.protocol.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.security.MessageDigest

object SecureChatManager {
    private lateinit var context: Context
    private lateinit var securePrefs: SharedPreferences
    private lateinit var cryptoManager: CryptoManager
    private var isInitialized = false
    private var isAuthenticated = false

    fun initialize(appContext: Context) {
        if (isInitialized) return
        context = appContext.applicationContext

        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        securePrefs = EncryptedSharedPreferences.create(
            context,
            "secure_chat_prefs",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )

        cryptoManager = CryptoManager()
        isInitialized = true
    }

    fun getContext(): Context = context

    fun hasAccount(): Boolean {
        return securePrefs.getBoolean("account_created", false)
    }

    fun isBiometricEnabled(): Boolean {
        return securePrefs.getBoolean("biometric_enabled", false)
    }

    fun setBiometricEnabled(enabled: Boolean) {
        securePrefs.edit().putBoolean("biometric_enabled", enabled).apply()
    }

    fun getStoredUsername(): String? {
        return securePrefs.getString("username", null)
    }

    private fun hashPassword(password: String, salt: String): String {
        val combined = "$salt$password"
        val digest = MessageDigest.getInstance("SHA-256")
        val hashBytes = digest.digest(combined.toByteArray())
        return bytesToBase64(hashBytes)
    }

    suspend fun createAccount(username: String, displayName: String, password: String): Boolean = withContext(Dispatchers.IO) {
        try {
            // Generate identity keys
            val identityKeyPair = cryptoManager.generateIdentityKeyPair()

            // Validate keys were generated
            if (identityKeyPair.publicKey.isEmpty() || identityKeyPair.secretKey.isEmpty()) {
                throw IllegalStateException("Failed to generate identity keys")
            }

            // Generate salt and hash password
            val salt = java.util.UUID.randomUUID().toString()
            val passwordHash = hashPassword(password, salt)

            // Store encrypted keys and credentials
            securePrefs.edit()
                .putString("identity_public_key", bytesToBase64(identityKeyPair.publicKey))
                .putString("identity_secret_key", bytesToBase64(identityKeyPair.secretKey))
                .putString("username", username)
                .putString("display_name", displayName)
                .putString("password_salt", salt)
                .putString("password_hash", passwordHash)
                .putBoolean("account_created", true)
                .apply()

            isAuthenticated = true
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }

    suspend fun unlockAccount(username: String, password: String): Boolean = withContext(Dispatchers.IO) {
        try {
            if (!hasAccount()) return@withContext false

            // Verify username
            val storedUsername = securePrefs.getString("username", null)
            if (storedUsername != username) return@withContext false

            // Verify password
            val salt = securePrefs.getString("password_salt", null) ?: return@withContext false
            val storedHash = securePrefs.getString("password_hash", null) ?: return@withContext false
            val inputHash = hashPassword(password, salt)

            if (storedHash != inputHash) return@withContext false

            isAuthenticated = true
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }

    suspend fun unlockWithBiometric(): Boolean = withContext(Dispatchers.IO) {
        try {
            if (!hasAccount()) return@withContext false
            if (!isBiometricEnabled()) return@withContext false

            isAuthenticated = true
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }

    fun getPublicKey(): String {
        return securePrefs.getString("identity_public_key", "") ?: ""
    }

    suspend fun getConversations(): List<Conversation> = withContext(Dispatchers.IO) {
        // TODO: Load from database
        emptyList()
    }

    suspend fun getMessages(conversationId: String): List<LocalMessage> = withContext(Dispatchers.IO) {
        // TODO: Load from database
        emptyList()
    }

    suspend fun getContact(contactId: String): Contact? = withContext(Dispatchers.IO) {
        // TODO: Load from database
        null
    }

    suspend fun getProfile(): UserProfile? = withContext(Dispatchers.IO) {
        val name = securePrefs.getString("display_name", null) ?: return@withContext null
        UserProfile(
            display_name = name,
            status_message = null,
            avatar = null,
            created_at = kotlinx.datetime.Clock.System.now()
        )
    }

    suspend fun sendMessage(conversationId: String, text: String): String = withContext(Dispatchers.IO) {
        // TODO: Encrypt and send message
        java.util.UUID.randomUUID().toString()
    }

    suspend fun addContact(name: String, publicKey: ByteArray): Contact = withContext(Dispatchers.IO) {
        val contact = Contact(
            id = java.util.UUID.randomUUID().toString(),
            display_name = name,
            public_key = publicKey,
            added_at = kotlinx.datetime.Clock.System.now(),
            last_seen = null,
            verified = false,
            blocked = false
        )
        // TODO: Store in database
        contact
    }

    suspend fun getOrCreateConversation(contactId: String): Conversation = withContext(Dispatchers.IO) {
        // TODO: Check if exists, create if not
        Conversation(
            id = java.util.UUID.randomUUID().toString(),
            contact_id = contactId,
            created_at = kotlinx.datetime.Clock.System.now(),
            updated_at = kotlinx.datetime.Clock.System.now(),
            last_message_preview = null,
            unread_count = 0,
            archived = false,
            pinned = false,
            ratchet_state = null
        )
    }

    private fun bytesToBase64(bytes: ByteArray): String {
        return android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
    }
}

class SecureChatApplication : android.app.Application() {
    override fun onCreate() {
        super.onCreate()
        SecureChatManager.initialize(this)
    }
}

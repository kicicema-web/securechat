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

    fun hasAccount(): Boolean {
        return securePrefs.getBoolean("account_created", false)
    }

    suspend fun createAccount(displayName: String, password: String): Boolean = withContext(Dispatchers.IO) {
        try {
            // Generate identity keys
            val identityKeyPair = cryptoManager.generateIdentityKeyPair()
            
            // Store encrypted keys
            securePrefs.edit()
                .putString("identity_public_key", bytesToBase64(identityKeyPair.publicKey))
                .putString("identity_secret_key", bytesToBase64(identityKeyPair.secretKey))
                .putString("display_name", displayName)
                .putBoolean("account_created", true)
                .apply()
            
            isAuthenticated = true
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }

    suspend fun unlockAccount(password: String): Boolean = withContext(Dispatchers.IO) {
        try {
            // In a real implementation, we'd use the password to decrypt stored keys
            // For now, just check if account exists
            if (!hasAccount()) return@withContext false
            
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

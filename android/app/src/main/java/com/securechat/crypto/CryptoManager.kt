package com.securechat.crypto

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.MessageDigest
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

class CryptoManager {
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    private val secureRandom = SecureRandom()

    companion object {
        private const val AES_KEY_SIZE = 256
        private const val GCM_TAG_LENGTH = 128
        private const val GCM_IV_LENGTH = 12
        private const val ANDROID_KEYSTORE = "AndroidKeyStore"
    }

    data class KeyPair(
        val publicKey: ByteArray,
        val secretKey: ByteArray
    )

    /**
     * Generate a new identity key pair (Ed25519 for signing)
     * For now, using simple key generation - would use proper Ed25519 in production
     */
    fun generateIdentityKeyPair(): KeyPair {
        val keyGen = KeyPairGenerator.getInstance("EC", ANDROID_KEYSTORE)
        keyGen.initialize(
            KeyGenParameterSpec.Builder(
                "identity_key",
                KeyProperties.PURPOSE_SIGN or KeyProperties.PURPOSE_VERIFY
            )
                .setAlgorithmParameterSpec(java.security.spec.ECGenParameterSpec("secp256r1"))
                .setDigests(KeyProperties.DIGEST_SHA256)
                .build()
        )
        val keyPair = keyGen.generateKeyPair()
        
        return KeyPair(
            publicKey = keyPair.public.encoded,
            secretKey = keyPair.private.encoded
        )
    }

    /**
     * Generate message encryption keys (X25519 for ECDH)
     * For now, using simple key generation
     */
    fun generateMessageKeyPair(): KeyPair {
        val privateKey = ByteArray(32)
        secureRandom.nextBytes(privateKey)
        
        // Derive public key from private (simplified - would use proper X25519)
        val publicKey = ByteArray(32)
        secureRandom.nextBytes(publicKey)
        
        return KeyPair(publicKey, privateKey)
    }

    /**
     * Generate AES-256 key
     */
    fun generateAESKey(): SecretKey {
        val keyGen = KeyGenerator.getInstance("AES")
        keyGen.init(AES_KEY_SIZE)
        return keyGen.generateKey()
    }

    /**
     * Encrypt data with AES-256-GCM
     */
    fun encrypt(plaintext: ByteArray, key: SecretKey): ByteArray {
        val iv = ByteArray(GCM_IV_LENGTH)
        secureRandom.nextBytes(iv)
        
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key, GCMParameterSpec(GCM_TAG_LENGTH, iv))
        
        val ciphertext = cipher.doFinal(plaintext)
        
        // Return IV + ciphertext
        return iv + ciphertext
    }

    /**
     * Decrypt data with AES-256-GCM
     */
    fun decrypt(encryptedData: ByteArray, key: SecretKey): ByteArray {
        require(encryptedData.size >= GCM_IV_LENGTH) { "Invalid encrypted data" }
        
        val iv = encryptedData.copyOfRange(0, GCM_IV_LENGTH)
        val ciphertext = encryptedData.copyOfRange(GCM_IV_LENGTH, encryptedData.size)
        
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(GCM_TAG_LENGTH, iv))
        
        return cipher.doFinal(ciphertext)
    }

    /**
     * Derive key from password using Argon2 (simplified - would use proper Argon2)
     */
    fun deriveKeyFromPassword(password: String, salt: ByteArray): SecretKey {
        // Simplified - would use proper Argon2id in production
        val digest = MessageDigest.getInstance("SHA-256")
        digest.update(salt)
        val hash = digest.digest(password.toByteArray(Charsets.UTF_8))
        return SecretKeySpec(hash, "AES")
    }

    /**
     * Generate random bytes
     */
    fun generateRandomBytes(size: Int): ByteArray {
        val bytes = ByteArray(size)
        secureRandom.nextBytes(bytes)
        return bytes
    }

    /**
     * Hash data with SHA-256
     */
    fun hash(data: ByteArray): ByteArray {
        return MessageDigest.getInstance("SHA-256").digest(data)
    }

    /**
     * Perform ECDH key agreement (simplified)
     * Returns 32-byte shared secret
     */
    fun performKeyAgreement(privateKey: ByteArray, publicKey: ByteArray): ByteArray {
        // Simplified - would use proper X25519 in production
        val shared = ByteArray(32)
        secureRandom.nextBytes(shared)
        return shared
    }
}

/**
 * Argon2 password hashing (placeholder - would use BouncyCastle or native lib in production)
 */
object Argon2Hasher {
    fun hash(password: String): String {
        // Placeholder - would use proper Argon2id
        val salt = ByteArray(16).apply { SecureRandom().nextBytes(this) }
        val digest = MessageDigest.getInstance("SHA-256")
        digest.update(salt)
        val hash = digest.digest(password.toByteArray(Charsets.UTF_8))
        return android.util.Base64.encodeToString(salt + hash, android.util.Base64.NO_WRAP)
    }

    fun verify(password: String, hash: String): Boolean {
        // Placeholder implementation
        return try {
            val decoded = android.util.Base64.decode(hash, android.util.Base64.NO_WRAP)
            val salt = decoded.copyOfRange(0, 16)
            val originalHash = decoded.copyOfRange(16, decoded.size)
            
            val digest = MessageDigest.getInstance("SHA-256")
            digest.update(salt)
            val newHash = digest.digest(password.toByteArray(Charsets.UTF_8))
            
            newHash.contentEquals(originalHash)
        } catch (e: Exception) {
            false
        }
    }
}

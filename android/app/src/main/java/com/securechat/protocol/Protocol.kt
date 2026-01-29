package com.securechat.protocol

import kotlinx.datetime.Instant
import kotlinx.serialization.Serializable

@Serializable
data class Contact(
    val id: String,
    val display_name: String,
    val public_key: ByteArray,
    val added_at: Instant,
    val last_seen: Instant?,
    val verified: Boolean,
    val blocked: Boolean
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false
        other as Contact
        return id == other.id
    }

    override fun hashCode(): Int = id.hashCode()
}

@Serializable
sealed class MessageContent {
    @Serializable
    data class Text(val text: String) : MessageContent()
    
    @Serializable
    data class Image(val data: ByteArray, val mime_type: String, val caption: String?) : MessageContent()
    
    @Serializable
    data class File(val data: ByteArray, val filename: String, val mime_type: String) : MessageContent()
    
    @Serializable
    data class Voice(val data: ByteArray, val duration_secs: Int) : MessageContent()
    
    @Serializable
    data class Location(val latitude: Double, val longitude: Double, val accuracy: Float?) : MessageContent()
}

@Serializable
data class LocalMessage(
    val id: String,
    val conversation_id: String,
    val sender_id: String,
    val is_outgoing: Boolean,
    val content: MessageContent,
    val timestamp: Instant,
    val sent: Boolean,
    val delivered: Boolean,
    val read: Boolean,
    val reply_to: String?
)

@Serializable
data class Conversation(
    val id: String,
    val contact_id: String,
    val created_at: Instant,
    val updated_at: Instant,
    val last_message_preview: String?,
    val unread_count: Int,
    val archived: Boolean,
    val pinned: Boolean,
    val ratchet_state: DoubleRatchetState?
)

@Serializable
data class DoubleRatchetState(
    val root_key: ByteArray,
    val sending_chain_key: ByteArray?,
    val receiving_chain_key: ByteArray?,
    val sending_message_number: Int,
    val receiving_message_number: Int
)

@Serializable
data class UserProfile(
    val display_name: String,
    val status_message: String?,
    val avatar: ByteArray?,
    val created_at: Instant
)

@Serializable
data class EncryptedMessage(
    val ciphertext: ByteArray,
    val nonce: ByteArray,
    val sender_pubkey: ByteArray,
    val ephemeral_pubkey: ByteArray
)

@Serializable
sealed class ProtocolMessage {
    @Serializable
    data class KeyBundle(
        val identity_key: ByteArray,
        val signed_prekey: ByteArray,
        val signed_prekey_signature: ByteArray,
        val one_time_prekeys: List<ByteArray>
    ) : ProtocolMessage()
    
    @Serializable
    data class Encrypted(
        val envelope: MessageEnvelope
    ) : ProtocolMessage()
    
    @Serializable
    data class DeliveryReceipt(
        val message_id: String,
        val timestamp: Instant
    ) : ProtocolMessage()
    
    @Serializable
    data class ReadReceipt(
        val message_id: String,
        val timestamp: Instant
    ) : ProtocolMessage()
    
    @Serializable
    data class Typing(
        val conversation_id: String,
        val is_typing: Boolean
    ) : ProtocolMessage()
    
    @Serializable
    data class ContactRequest(
        val display_name: String,
        val message: String,
        val key_bundle: KeyBundle
    ) : ProtocolMessage()
    
    @Serializable
    data class ContactResponse(
        val accepted: Boolean,
        val key_bundle: KeyBundle?
    ) : ProtocolMessage()
}

@Serializable
data class MessageEnvelope(
    val id: String,
    val sender_id: String,
    val recipient_id: String,
    val timestamp: Instant,
    val encrypted_content: EncryptedMessage,
    val signature: ByteArray,
    val reply_to: String?
)

package com.securechat.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.securechat.SecureChatManager
import com.securechat.protocol.Contact
import com.securechat.protocol.Conversation
import com.securechat.protocol.LocalMessage
import com.securechat.protocol.UserProfile
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class ChatViewModel : ViewModel() {
    private val _conversations = MutableStateFlow<List<Conversation>>(emptyList())
    val conversations: StateFlow<List<Conversation>> = _conversations

    private val _messages = MutableStateFlow<List<LocalMessage>>(emptyList())
    val messages: StateFlow<List<LocalMessage>> = _messages

    private val _currentConversation = MutableStateFlow<Conversation?>(null)
    val currentConversation: StateFlow<Conversation?> = _currentConversation

    private val _currentContact = MutableStateFlow<Contact?>(null)
    val currentContact: StateFlow<Contact?> = _currentContact

    private val _userProfile = MutableStateFlow<UserProfile?>(null)
    val userProfile: StateFlow<UserProfile?> = _userProfile

    init {
        loadConversations()
        loadUserProfile()
    }

    fun loadConversations() {
        viewModelScope.launch {
            try {
                _conversations.value = SecureChatManager.getConversations()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun loadUserProfile() {
        viewModelScope.launch {
            try {
                _userProfile.value = SecureChatManager.getProfile()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun selectConversation(conversation: Conversation) {
        _currentConversation.value = conversation
        viewModelScope.launch {
            try {
                _currentContact.value = SecureChatManager.getContact(conversation.contact_id)
                loadMessages(conversation.id)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun loadMessages(conversationId: String) {
        viewModelScope.launch {
            try {
                _messages.value = SecureChatManager.getMessages(conversationId)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun sendMessage(text: String) {
        val conv = _currentConversation.value ?: return
        viewModelScope.launch {
            try {
                SecureChatManager.sendMessage(conv.id, text)
                loadMessages(conv.id)
                loadConversations()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun addContact(name: String, publicKeyBase64: String) {
        viewModelScope.launch {
            try {
                val publicKey = android.util.Base64.decode(publicKeyBase64, android.util.Base64.DEFAULT)
                val contact = SecureChatManager.addContact(name, publicKey)
                SecureChatManager.getOrCreateConversation(contact.id)
                loadConversations()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun getPublicKey(): String {
        return SecureChatManager.getPublicKey()
    }
}

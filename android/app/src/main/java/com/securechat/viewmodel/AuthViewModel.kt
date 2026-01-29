package com.securechat.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.securechat.SecureChatManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class AuthViewModel : ViewModel() {
    private val _uiState = MutableStateFlow<UiState>(UiState.Idle)
    val uiState: StateFlow<UiState> = _uiState

    fun createAccount(displayName: String, password: String) {
        viewModelScope.launch {
            _uiState.value = UiState.Loading
            try {
                val result = SecureChatManager.createAccount(displayName, password)
                _uiState.value = if (result) UiState.Success else UiState.Error("Failed to create account")
            } catch (e: Exception) {
                _uiState.value = UiState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun unlockAccount(password: String) {
        viewModelScope.launch {
            _uiState.value = UiState.Loading
            try {
                val result = SecureChatManager.unlockAccount(password)
                _uiState.value = if (result) UiState.Success else UiState.Error("Invalid password")
            } catch (e: Exception) {
                _uiState.value = UiState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun hasAccount(): Boolean {
        return SecureChatManager.hasAccount()
    }

    sealed class UiState {
        object Idle : UiState()
        object Loading : UiState()
        object Success : UiState()
        data class Error(val message: String) : UiState()
    }
}

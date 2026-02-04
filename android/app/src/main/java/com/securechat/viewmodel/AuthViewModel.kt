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

    fun createAccount(username: String, displayName: String, password: String) {
        viewModelScope.launch {
            _uiState.value = UiState.Loading
            try {
                val result = SecureChatManager.createAccount(username, displayName, password)
                _uiState.value = if (result) UiState.Success else UiState.Error("Failed to create account")
            } catch (e: Exception) {
                _uiState.value = UiState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun unlockAccount(username: String, password: String) {
        viewModelScope.launch {
            _uiState.value = UiState.Loading
            try {
                val result = SecureChatManager.unlockAccount(username, password)
                _uiState.value = if (result) UiState.Success else UiState.Error("Invalid username or password")
            } catch (e: Exception) {
                _uiState.value = UiState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun unlockWithBiometric() {
        viewModelScope.launch {
            _uiState.value = UiState.Loading
            try {
                val result = SecureChatManager.unlockWithBiometric()
                _uiState.value = if (result) UiState.Success else UiState.Error("Biometric authentication failed")
            } catch (e: Exception) {
                _uiState.value = UiState.Error(e.message ?: "Unknown error")
            }
        }
    }

    fun hasAccount(): Boolean {
        return SecureChatManager.hasAccount()
    }

    fun isBiometricEnabled(): Boolean {
        return SecureChatManager.isBiometricEnabled()
    }

    fun setBiometricEnabled(enabled: Boolean) {
        SecureChatManager.setBiometricEnabled(enabled)
    }

    fun getStoredUsername(): String? {
        return SecureChatManager.getStoredUsername()
    }

    sealed class UiState {
        object Idle : UiState()
        object Loading : UiState()
        object Success : UiState()
        data class Error(val message: String) : UiState()
    }
}

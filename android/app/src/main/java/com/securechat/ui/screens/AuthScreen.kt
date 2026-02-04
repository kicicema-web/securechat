package com.securechat.ui.screens

import android.content.Context
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Fingerprint
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.securechat.viewmodel.AuthViewModel

@Composable
fun AuthScreen(
    viewModel: AuthViewModel,
    onNavigateToChat: () -> Unit
) {
    var isCreateAccount by remember { mutableStateOf(false) }
    var username by remember { mutableStateOf("") }
    var displayName by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var confirmPassword by remember { mutableStateOf("") }
    var passwordVisible by remember { mutableStateOf(false) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var isLoading by remember { mutableStateOf(false) }
    var enableBiometric by remember { mutableStateOf(false) }

    val context = LocalContext.current
    val uiState by viewModel.uiState.collectAsState()

    // Check if biometric is available
    val biometricManager = remember { BiometricManager.from(context) }
    val canUseBiometric = remember {
        biometricManager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) ==
            BiometricManager.BIOMETRIC_SUCCESS
    }
    val isBiometricEnabled = remember { viewModel.isBiometricEnabled() }
    val storedUsername = remember { viewModel.getStoredUsername() }

    // Pre-fill username if stored
    LaunchedEffect(storedUsername) {
        if (storedUsername != null && !isCreateAccount) {
            username = storedUsername
        }
    }

    LaunchedEffect(uiState) {
        when (uiState) {
            is AuthViewModel.UiState.Success -> {
                if (enableBiometric) {
                    viewModel.setBiometricEnabled(true)
                }
                onNavigateToChat()
            }
            is AuthViewModel.UiState.Error -> {
                errorMessage = (uiState as AuthViewModel.UiState.Error).message
                isLoading = false
            }
            is AuthViewModel.UiState.Loading -> isLoading = true
            else -> isLoading = false
        }
    }

    fun showBiometricPrompt() {
        val activity = context as? FragmentActivity ?: return
        val executor = ContextCompat.getMainExecutor(context)

        val biometricPrompt = BiometricPrompt(
            activity,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    viewModel.unlockWithBiometric()
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    errorMessage = errString.toString()
                }

                override fun onAuthenticationFailed() {
                    errorMessage = "Biometric authentication failed"
                }
            }
        )

        val promptInfo = BiometricPrompt.PromptInfo.Builder()
            .setTitle("SecureChat Login")
            .setSubtitle("Use your fingerprint to unlock")
            .setNegativeButtonText("Use password instead")
            .build()

        biometricPrompt.authenticate(promptInfo)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp)
            .verticalScroll(rememberScrollState()),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        // Logo
        Box(
            modifier = Modifier
                .size(80.dp)
                .background(
                    brush = androidx.compose.ui.graphics.Brush.linearGradient(
                        colors = listOf(
                            MaterialTheme.colorScheme.primary,
                            MaterialTheme.colorScheme.primaryContainer
                        )
                    ),
                    shape = MaterialTheme.shapes.large
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "SC",
                style = MaterialTheme.typography.headlineLarge.copy(
                    color = MaterialTheme.colorScheme.onPrimary,
                    fontWeight = FontWeight.Bold
                )
            )
        }

        Spacer(modifier = Modifier.height(24.dp))

        Text(
            text = "SecureChat",
            style = MaterialTheme.typography.headlineMedium.copy(
                fontWeight = FontWeight.SemiBold
            )
        )

        Text(
            text = "End-to-end encrypted messaging",
            style = MaterialTheme.typography.bodyMedium.copy(
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        )

        Spacer(modifier = Modifier.height(32.dp))

        // Error message
        errorMessage?.let { error ->
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer
                ),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = error,
                    color = MaterialTheme.colorScheme.onErrorContainer,
                    modifier = Modifier.padding(16.dp)
                )
            }
            Spacer(modifier = Modifier.height(16.dp))
        }

        // Username field (for both login and create account)
        OutlinedTextField(
            value = username,
            onValueChange = { username = it },
            label = { Text("Username") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
            leadingIcon = {
                Icon(Icons.Default.Person, contentDescription = "Username")
            }
        )
        Spacer(modifier = Modifier.height(16.dp))

        // Display name field (only for create account)
        if (isCreateAccount) {
            OutlinedTextField(
                value = displayName,
                onValueChange = { displayName = it },
                label = { Text("Display Name") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next)
            )
            Spacer(modifier = Modifier.height(16.dp))
        }

        // Password field
        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") },
            singleLine = true,
            visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Password,
                imeAction = if (isCreateAccount) ImeAction.Next else ImeAction.Done
            ),
            keyboardActions = KeyboardActions(
                onDone = {
                    if (!isCreateAccount) {
                        viewModel.unlockAccount(username, password)
                    }
                }
            ),
            trailingIcon = {
                IconButton(onClick = { passwordVisible = !passwordVisible }) {
                    Icon(
                        imageVector = if (passwordVisible) Icons.Default.Visibility else Icons.Default.VisibilityOff,
                        contentDescription = if (passwordVisible) "Hide password" else "Show password"
                    )
                }
            },
            modifier = Modifier.fillMaxWidth()
        )

        // Confirm password field (only for create account)
        if (isCreateAccount) {
            Spacer(modifier = Modifier.height(16.dp))
            OutlinedTextField(
                value = confirmPassword,
                onValueChange = { confirmPassword = it },
                label = { Text("Confirm Password") },
                singleLine = true,
                visualTransformation = if (passwordVisible) VisualTransformation.None else PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Password,
                    imeAction = ImeAction.Done
                ),
                modifier = Modifier.fillMaxWidth()
            )

            // Enable biometric checkbox
            if (canUseBiometric) {
                Spacer(modifier = Modifier.height(16.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Checkbox(
                        checked = enableBiometric,
                        onCheckedChange = { enableBiometric = it }
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("Enable fingerprint login")
                }
            }
        }

        Spacer(modifier = Modifier.height(24.dp))

        // Action button
        Button(
            onClick = {
                errorMessage = null
                if (isCreateAccount) {
                    when {
                        username.isBlank() -> errorMessage = "Please enter a username"
                        username.length < 3 -> errorMessage = "Username must be at least 3 characters"
                        displayName.isBlank() -> errorMessage = "Please enter a display name"
                        password.length < 8 -> errorMessage = "Password must be at least 8 characters"
                        password != confirmPassword -> errorMessage = "Passwords do not match"
                        else -> viewModel.createAccount(username, displayName, password)
                    }
                } else {
                    when {
                        username.isBlank() -> errorMessage = "Please enter your username"
                        password.isBlank() -> errorMessage = "Please enter your password"
                        else -> viewModel.unlockAccount(username, password)
                    }
                }
            },
            modifier = Modifier
                .fillMaxWidth()
                .height(50.dp),
            enabled = !isLoading
        ) {
            if (isLoading) {
                CircularProgressIndicator(
                    modifier = Modifier.size(24.dp),
                    color = MaterialTheme.colorScheme.onPrimary
                )
            } else {
                Text(if (isCreateAccount) "Create Account" else "Sign In")
            }
        }

        // Biometric login button (only for login when biometric is enabled)
        if (!isCreateAccount && canUseBiometric && isBiometricEnabled && viewModel.hasAccount()) {
            Spacer(modifier = Modifier.height(16.dp))
            OutlinedButton(
                onClick = { showBiometricPrompt() },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(50.dp),
                enabled = !isLoading
            ) {
                Icon(
                    Icons.Default.Fingerprint,
                    contentDescription = "Fingerprint",
                    modifier = Modifier.size(24.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Use Fingerprint")
            }
        }

        Spacer(modifier = Modifier.height(16.dp))

        // Toggle between login and create account
        TextButton(
            onClick = {
                isCreateAccount = !isCreateAccount
                errorMessage = null
                password = ""
                confirmPassword = ""
                if (!isCreateAccount && storedUsername != null) {
                    username = storedUsername
                }
            }
        ) {
            Text(
                if (isCreateAccount) "Already have an account? Sign in"
                else "Need an account? Create one"
            )
        }
    }
}

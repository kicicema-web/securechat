package com.securechat

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.securechat.ui.screens.*
import com.securechat.ui.theme.SecureChatTheme
import com.securechat.viewmodel.AuthViewModel
import com.securechat.viewmodel.ChatViewModel

class MainActivity : ComponentActivity() {
    private val authViewModel: AuthViewModel by viewModels()
    private val chatViewModel: ChatViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        setContent {
            SecureChatTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    SecureChatApp(
                        authViewModel = authViewModel,
                        chatViewModel = chatViewModel
                    )
                }
            }
        }
    }
}

@Composable
fun SecureChatApp(
    authViewModel: AuthViewModel,
    chatViewModel: ChatViewModel
) {
    val navController = rememberNavController()
    
    NavHost(navController = navController, startDestination = "splash") {
        composable("splash") {
            SplashScreen(
                onNavigateToAuth = { navController.navigate("auth") { popUpTo("splash") { inclusive = true } } },
                onNavigateToChat = { navController.navigate("chat") { popUpTo("splash") { inclusive = true } } }
            )
        }
        
        composable("auth") {
            AuthScreen(
                viewModel = authViewModel,
                onNavigateToChat = { navController.navigate("chat") { popUpTo("auth") { inclusive = true } } }
            )
        }
        
        composable("chat") {
            ChatScreen(
                viewModel = chatViewModel,
                onNavigateToAuth = { navController.navigate("auth") { popUpTo("chat") { inclusive = true } } }
            )
        }
    }
}

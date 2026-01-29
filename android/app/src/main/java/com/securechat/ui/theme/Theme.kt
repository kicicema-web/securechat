package com.securechat.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

// Dark theme colors - matching desktop app
private val DarkColorScheme = darkColorScheme(
    primary = Color(0xFF58A6FF),
    onPrimary = Color.White,
    primaryContainer = Color(0xFF1F6FEB),
    onPrimaryContainer = Color.White,
    secondary = Color(0xFF8B949E),
    onSecondary = Color(0xFF0D1117),
    secondaryContainer = Color(0xFF21262D),
    onSecondaryContainer = Color(0xFFF0F6FC),
    tertiary = Color(0xFF238636),
    onTertiary = Color.White,
    tertiaryContainer = Color(0xFF238636),
    onTertiaryContainer = Color.White,
    error = Color(0xFFF85149),
    onError = Color.White,
    errorContainer = Color(0xFFF85149),
    onErrorContainer = Color.White,
    background = Color(0xFF0D1117),
    onBackground = Color(0xFFF0F6FC),
    surface = Color(0xFF161B22),
    onSurface = Color(0xFFF0F6FC),
    surfaceVariant = Color(0xFF21262D),
    onSurfaceVariant = Color(0xFF8B949E),
    outline = Color(0xFF30363D),
    inverseSurface = Color(0xFFF0F6FC),
    inverseOnSurface = Color(0xFF0D1117),
    inversePrimary = Color(0xFF58A6FF),
    surfaceTint = Color(0xFF58A6FF),
    outlineVariant = Color(0xFF30363D),
    scrim = Color(0xFF000000),
)

@Composable
fun SecureChatTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> DarkColorScheme // Always use dark theme for consistent look
    }
    
    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            window.statusBarColor = colorScheme.background.toArgb()
            WindowCompat.getInsetsController(window, view).isAppearanceLightStatusBars = false
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}

package com.wfhub.cagatayldzz.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import com.wfhub.cagatayldzz.data.Settings

val LocalSettings = compositionLocalOf<Settings> { error("Settings are provided by WFHubApp") }

/** Same palette as `WFStyle` of the iOS app. */
object WF {
  val gold = Color(0xFFE0C773)
  val goldDark = Color(0xFF735409)
  val artwork = Brush.verticalGradient(listOf(Color(0xFF141417), Color(0xFF262E36), Color.Black))
}

private val Dark = darkColorScheme(primary = WF.gold, onPrimary = Color.Black, background = Color(0xFF121214), surface = Color(0xFF121214), surfaceVariant = Color(0xFF1F1F23))
private val Light = lightColorScheme(primary = WF.goldDark, onPrimary = Color.White, background = Color(0xFFF7F7F9), surface = Color(0xFFF7F7F9), surfaceVariant = Color(0xFFEAEAEF))

@Composable
fun WFHubTheme(settings: Settings, content: @Composable () -> Unit) {
  val dark = when (settings.theme) {
    "dark" -> true
    "light" -> false
    else -> isSystemInDarkTheme()
  }
  CompositionLocalProvider(LocalSettings provides settings) {
    MaterialTheme(colorScheme = if (dark) Dark else Light, content = content)
  }
}

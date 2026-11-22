package com.wfhub.cagatayldzz.data

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import java.util.Locale
import uniffi.wfhub_core.resolveLanguage

/** The preferences of the iOS app (`tennoName`, `platform`, `appLanguage`, `appTheme`), kept in SharedPreferences. */
class Settings(context: Context) {
  private val prefs = context.getSharedPreferences("wfhub", Context.MODE_PRIVATE)

  var tennoName by mutableStateOf(prefs.getString("tennoName", "Tenno") ?: "Tenno")
    private set
  var platform by mutableStateOf(prefs.getString("platform", "pc") ?: "pc")
    private set
  /** `system` or a language code. */
  var language by mutableStateOf(prefs.getString("appLanguage", "system") ?: "system")
    private set
  /** `system`, `light` or `dark`. */
  var theme by mutableStateOf(prefs.getString("appTheme", "system") ?: "system")
    private set

  /** The language the texts are shown in: the choice, or the first supported language of the device. */
  val languageCode: String
    get() = if (language == "system") resolveLanguage(deviceLanguages()) else language

  fun saveProfile(name: String, platform: String) {
    tennoName = name.trim().ifEmpty { "Tenno" }
    this.platform = platform
    prefs.edit().putString("tennoName", tennoName).putString("platform", platform).apply()
  }

  fun setLanguage(value: String) {
    language = value
    prefs.edit().putString("appLanguage", value).apply()
  }

  fun setTheme(value: String) {
    theme = value
    prefs.edit().putString("appTheme", value).apply()
  }

  private fun deviceLanguages(): List<String> {
    val locales = android.content.res.Resources.getSystem().configuration.locales
    return (0 until locales.size()).map { locales[it].let { l -> l.language + "-" + l.country } } + Locale.getDefault().language
  }
}

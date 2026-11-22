package com.wfhub.cagatayldzz.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import uniffi.wfhub_core.translate
import uniffi.wfhub_core.translateData
import uniffi.wfhub_core.translateFormat

// Texts come from the Rust core, which embeds the same `Localizable.strings` as the iOS app (14 languages).

/** UI text by key, e.g. `t("retry")`. */
@Composable
fun t(key: String): String {
  val language = LocalSettings.current.languageCode
  return remember(language, key) { translate(language, key) }
}

/** Text with arguments, e.g. `tf("seasonNumber", 7)`. */
@Composable
fun tf(key: String, vararg arguments: Any): String {
  val language = LocalSettings.current.languageCode
  return remember(language, key, arguments.toList()) { translateFormat(language, key, arguments.map { it.toString() }) }
}

/** Text that comes from the API (English). */
@Composable
fun td(text: String): String {
  val language = LocalSettings.current.languageCode
  return remember(language, text) { translateData(language, text) }
}

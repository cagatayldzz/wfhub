package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.wfhub.cagatayldzz.data.Http
import com.wfhub.cagatayldzz.ui.*
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import uniffi.wfhub_core.Choice
import uniffi.wfhub_core.feedbackErrorKey
import uniffi.wfhub_core.feedbackIsValid
import uniffi.wfhub_core.feedbackRequest
import uniffi.wfhub_core.languages
import uniffi.wfhub_core.platforms
import uniffi.wfhub_core.publicUrl

private const val SITE = "https://wfhub.cagatayldzz.com"

@Composable
fun SettingsScreen(nav: NavController) {
  val settings = LocalSettings.current
  var name by rememberSaveable { mutableStateOf(settings.tennoName) }
  var platform by rememberSaveable { mutableStateOf(settings.platform) }
  var saved by remember { mutableStateOf(false) }
  val uri = LocalUriHandler.current
  val platformChoices = platforms()

  Column(Modifier.verticalScroll(rememberScrollState()).padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
    ScreenTitle(t("settings"))
    Text(t("profile"), style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    OutlinedTextField(name, { name = it }, Modifier.fillMaxWidth(), singleLine = true, label = { Text(t("tennoName")) })
    Picker(t("platform"), platformChoices.firstOrNull { it.code == platform }?.title ?: "PC", platformChoices) { platform = it }
    Button(onClick = { settings.saveProfile(name, platform); saved = true }, Modifier.fillMaxWidth()) { Text(t("save")) }

    HorizontalDivider()
    Text(t("preferences"), style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    val languageChoices = listOf(Choice("system", t("system"))) + languages()
    Picker(t("language"), languageChoices.firstOrNull { it.code == settings.language }?.title ?: "English", languageChoices) { settings.setLanguage(it) }
    val themeChoices = listOf(Choice("system", t("system")), Choice("light", t("light")), Choice("dark", t("dark")))
    Picker(t("appearance"), themeChoices.firstOrNull { it.code == settings.theme }?.title ?: t("system"), themeChoices) { settings.setTheme(it) }

    HorizontalDivider()
    LinkRow(t("feedback")) { nav.navigate("feedback") }
    LinkRow(t("privacyPolicy")) { uri.openUri("$SITE/legal/privacy-policy") }
  }
  if (saved) {
    AlertDialog(onDismissRequest = { saved = false }, confirmButton = { TextButton(onClick = { saved = false }) { Text(t("ok")) } }, title = { Text(t("settingsSaved")) })
  }
}

@Composable
private fun LinkRow(title: String, onClick: () -> Unit) {
  Text(title, fontWeight = FontWeight.SemiBold, modifier = Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 12.dp))
}

/** A labelled value that opens a menu of choices. */
@Composable
private fun Picker(label: String, value: String, choices: List<Choice>, onPick: (String) -> Unit) {
  var open by remember { mutableStateOf(false) }
  Column {
    Text(label, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
    Text(value, fontWeight = FontWeight.SemiBold, modifier = Modifier.fillMaxWidth().clickable { open = true }.padding(vertical = 8.dp))
    DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
      choices.forEach { choice -> DropdownMenuItem(text = { Text(choice.title) }, onClick = { onPick(choice.code); open = false }) }
    }
  }
}

@Composable
fun FeedbackScreen(nav: NavController) {
  var title by rememberSaveable { mutableStateOf("") }
  var message by rememberSaveable { mutableStateOf("") }
  var category by rememberSaveable { mutableStateOf("bug") }
  var sending by remember { mutableStateOf(false) }
  var sent by remember { mutableStateOf(false) }
  var error by remember { mutableStateOf<String?>(null) }
  val scope = rememberCoroutineScope()
  val valid = feedbackIsValid(title, message)

  DetailScreen(nav, t("feedback")) {
    Column(Modifier.verticalScroll(rememberScrollState()).padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
      Text(t("feedbackIntro"), color = MaterialTheme.colorScheme.onSurfaceVariant)
      Text(t("category"), style = MaterialTheme.typography.labelLarge)
      Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        listOf("bug" to "bugReport", "suggestion" to "suggestion", "other" to "other").forEach { (code, key) ->
          FilterChip(selected = category == code, onClick = { category = code }, label = { Text(t(key)) })
        }
      }
      OutlinedTextField(title, { title = it }, Modifier.fillMaxWidth(), singleLine = true, label = { Text(t("title")) }, supportingText = { Text(t("characters3To120")) })
      OutlinedTextField(message, { message = it }, Modifier.fillMaxWidth(), minLines = 6, label = { Text(t("message")) }, supportingText = { Text(t("characters10To4000")) })
      error?.let { Text(t(it), color = MaterialTheme.colorScheme.error) }
      Button(
        enabled = valid && !sending,
        modifier = Modifier.fillMaxWidth(),
        onClick = {
          sending = true
          error = null
          scope.launch {
            try {
              val request = feedbackRequest(title, category, message)
              val status = Http.post(request.url, request.body)
              feedbackErrorKey(status.toUShort())?.let { error = it } ?: run { sent = true }
            } catch (e: CancellationException) {
              throw e
            } catch (e: Exception) {
              error = "feedbackSendFailed"
            } finally {
              sending = false
            }
          }
        },
      ) { Text(t("sendFeedback")) }
    }
  }
  if (sent) {
    AlertDialog(
      onDismissRequest = { sent = false; nav.popBackStack() },
      confirmButton = { TextButton(onClick = { sent = false; nav.popBackStack() }) { Text(t("ok")) } },
      title = { Text(t("feedbackThanks")) },
    )
  }
}

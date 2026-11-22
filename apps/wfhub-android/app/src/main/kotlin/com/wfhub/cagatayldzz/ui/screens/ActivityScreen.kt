package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.wfhub.cagatayldzz.data.Http
import com.wfhub.cagatayldzz.data.load
import com.wfhub.cagatayldzz.ui.*
import uniffi.wfhub_core.ActivityEntry
import uniffi.wfhub_core.ActivityPage
import uniffi.wfhub_core.BuildProgress
import uniffi.wfhub_core.PageHero
import uniffi.wfhub_core.SubRow
import uniffi.wfhub_core.activityRoute
import uniffi.wfhub_core.activityUrl
import uniffi.wfhub_core.filterEntries
import uniffi.wfhub_core.parseActivity

/** One world state endpoint: the Rust core reads the JSON into an `ActivityPage`, this only draws it. */
@Composable
fun ActivityScreen(nav: NavController, endpoint: String) {
  val settings = LocalSettings.current
  val route = activityRoute(endpoint, endpoint)
  val state = load(settings.platform to endpoint, refreshSeconds = 60) {
    parseActivity(endpoint, Http.get(activityUrl(settings.platform, endpoint)))
  }
  DetailScreen(nav, td(route.title)) { LoadedContent(state) { page -> ActivityPageView(page) } }
}

@Composable
private fun ActivityPageView(page: ActivityPage) {
  var query by rememberSaveable { mutableStateOf("") }
  var tier by rememberSaveable { mutableStateOf("All") }
  val entries = filterEntries(page.entries, query, tier)
  LazyColumn(contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
    page.hero?.let { hero -> item { Hero(hero) } }
    items(page.progress) { ProgressCard(it) }
    if (page.tiers.isNotEmpty()) {
      item {
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
          items(page.tiers) { name -> FilterChip(selected = tier == name, onClick = { tier = name }, label = { Text(td(if (name == "All") "All" else name.uppercase())) }) }
        }
      }
    }
    if (page.searchable) {
      item { OutlinedTextField(query, { query = it }, Modifier.fillMaxWidth(), singleLine = true, placeholder = { Text(t("search")) }) }
    }
    if (entries.isEmpty() && page.progress.isEmpty()) item { Text(t("noDataFound"), color = MaterialTheme.colorScheme.onSurfaceVariant) }
    items(entries) { EntryCard(it) }
  }
}

@Composable
private fun Hero(hero: PageHero) {
  Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp))) {
    Artwork(hero.image, Modifier.matchParentSize(), ContentScale.Crop)
    Box(Modifier.matchParentSize().background(Color.Black.copy(alpha = 0.5f)))
    Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
      Text(t("nightwave"), fontSize = 26.sp, fontWeight = FontWeight.Black, color = WF.gold)
      hero.season?.let { Text(tf("seasonNumber", it), color = Color.White) }
      Countdown(hero.expiry, color = Color.White)
    }
  }
}

@Composable
private fun ProgressCard(progress: BuildProgress) {
  WFCard {
    Row {
      Column(Modifier.weight(1f)) {
        Text(progress.title, fontWeight = FontWeight.Bold)
        Text(progress.faction, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
      Text(progress.percent?.let { "%.2f%%".format(it) } ?: t("unavailable"), fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary)
    }
    progress.percent?.let { LinearProgressIndicator(progress = { (it / 100).toFloat() }, Modifier.fillMaxWidth().padding(top = 6.dp)) }
  }
}

@Composable
private fun EntryCard(entry: ActivityEntry) {
  val uri = LocalUriHandler.current
  WFCard(onClick = entry.link?.let { link -> { uri.openUri(link) } }) {
    entry.image?.let { Artwork(it, Modifier.fillMaxWidth().height(150.dp).clip(RoundedCornerShape(5.dp)), ContentScale.Crop) }
    Row {
      Text(td(entry.title), fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
      entry.badge?.let { Text(it, fontSize = 12.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary) }
    }
    entry.detail?.let { Text(td(it), color = MaterialTheme.colorScheme.onSurfaceVariant) }
    if (isFuture(entry.expiry)) Countdown(entry.expiry, color = MaterialTheme.colorScheme.onSurfaceVariant)
    entry.fields.forEach { field ->
      Row {
        Text(td(field.label), color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f))
        Text(field.value, fontWeight = FontWeight.Medium)
      }
    }
    entry.sections.forEach { SubRowView(it) }
    entry.notes.forEach { Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant) }
  }
}

@Composable
private fun SubRowView(row: SubRow) {
  HorizontalDivider(Modifier.padding(vertical = 4.dp))
  Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
    row.image?.let { Artwork(it, Modifier.size(44.dp)) }
    Column(Modifier.weight(1f)) {
      Text(row.title, fontWeight = FontWeight.SemiBold)
      row.kind?.let { Text(td(it), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.primary) }
      if (row.detail.isNotEmpty()) Text(row.detail, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
    row.trailing?.let { Text(it, fontSize = 12.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary) }
  }
}

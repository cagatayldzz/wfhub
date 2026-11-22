package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import uniffi.wfhub_core.AlertCard
import uniffi.wfhub_core.EventCard
import uniffi.wfhub_core.HomeBanner
import uniffi.wfhub_core.HomeResponse
import uniffi.wfhub_core.NewsCard
import uniffi.wfhub_core.categoryImageUrl
import uniffi.wfhub_core.expiryUnix
import uniffi.wfhub_core.homeRequests
import uniffi.wfhub_core.parseHome
import uniffi.wfhub_core.publicUrl

@Composable
fun HomeScreen(nav: NavController) {
  val settings = LocalSettings.current
  val language = settings.languageCode
  val state = load(settings.platform to language, refreshSeconds = 60) {
    val responses = coroutineScope {
      homeRequests(settings.platform).map { request -> async { HomeResponse(request.key, Http.getOrNull(request.url)) } }.awaitAll()
    }
    parseHome(responses, language)
  }
  val uri = LocalUriHandler.current

  LoadedContent(state) { home ->
    LazyColumn(Modifier.fillMaxWidth(), contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp)) {
      item { Text("${t("welcome")}, ${settings.tennoName}", fontSize = 28.sp, fontWeight = FontWeight.ExtraBold, modifier = Modifier.padding(top = 24.dp, bottom = 12.dp)) }
      items(home.banners.size) { index -> home.banners[index].let { Banner(it.image, it.title, null, null) { uri.openUri(it.link) } } }
      home.nightwave?.let { nightwave ->
        item {
          WFCard(onClick = { nav.navigate(Routes.activity("nightwave")) }) {
            Text(t("nightwave"), fontSize = 26.sp, fontWeight = FontWeight.Black, color = MaterialTheme.colorScheme.primary)
            Text(tf("seasonNumber", nightwave.season ?: 0))
            Countdown(nightwave.expiry)
          }
        }
      }
      if (home.events.isNotEmpty()) {
        item { SectionHeader(t("events"), t("latestEvents")) { nav.navigate(Routes.activity("events")) } }
        items(home.events.size) { EventRow(home.events[it]) }
      }
      if (home.news.isNotEmpty()) {
        item { SectionHeader(t("news"), t("latestNews")) { nav.navigate(Routes.activity("news")) } }
        items(home.news.size) { NewsRow(home.news[it], it + 1) { link -> uri.openUri(link) } }
      }
      home.trader?.let { trader ->
        item {
          Banner(publicUrl("sections/baro-ki-teer.png"), "Baro Ki'Teer", trader.location, trader.expiry) { nav.navigate(Routes.activity("voidTrader")) }
        }
      }
      if (home.alerts.isNotEmpty()) {
        item { SectionHeader(t("activities"), t("latestActivities")) { nav.navigate(Routes.activity("alerts")) } }
        items(home.alerts.size) { AlertRow(home.alerts[it]) }
      }
      item {
        SectionHeader(t("codex"), t("codexContent")) { nav.navigate("catalogue") }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
          for (category in listOf("warframes", "secondary")) {
            Box(Modifier.weight(1f)) { ArtworkCard(t(category), categoryImageUrl(category)) { nav.navigate(Routes.category(category)) } }
          }
        }
      }
      if (home.failed.isNotEmpty()) {
        item {
          Column(Modifier.fillMaxWidth().padding(vertical = 24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(t("someContentFailedToLoad"), color = MaterialTheme.colorScheme.onSurfaceVariant)
            Button(onClick = state.reload) { Text(t("retry")) }
          }
        }
      }
    }
  }
}

@Composable
private fun Banner(image: String?, title: String, subtitle: String?, expiry: String?, onClick: () -> Unit) {
  Box(Modifier.padding(vertical = 6.dp).fillMaxWidth().height(110.dp).clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick)) {
    Artwork(image, Modifier.matchParentSize(), ContentScale.Crop)
    Box(Modifier.matchParentSize().background(Color.Black.copy(alpha = 0.5f)))
    Column(Modifier.align(Alignment.CenterStart).padding(horizontal = 20.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
      Text(title.uppercase(), fontSize = 22.sp, fontWeight = FontWeight.Black, color = WF.gold)
      if (!subtitle.isNullOrEmpty()) Text(subtitle, color = Color.White)
      if (expiry != null && expiryUnix(expiry) != null) Countdown(expiry, color = Color.White)
    }
  }
}

@Composable
private fun EventRow(event: EventCard) {
  WFCard(Modifier.padding(vertical = 6.dp)) {
    Row {
      Text(event.title, fontWeight = FontWeight.Bold, maxLines = 2, modifier = Modifier.weight(1f))
      event.progress?.let { Text("%.2f%%".format(it), fontWeight = FontWeight.Bold, fontSize = 12.sp) }
    }
    Row {
      Text(event.node, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f))
      Countdown(event.expiry)
    }
  }
}

@Composable
private fun AlertRow(alert: AlertCard) {
  WFCard(Modifier.padding(vertical = 6.dp)) {
    Text(alert.title, fontWeight = FontWeight.Bold)
    Row {
      Text(alert.rewards, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f))
      Countdown(alert.expiry)
    }
  }
}

@Composable
private fun NewsRow(item: NewsCard, index: Int, open: (String) -> Unit) {
  Column(
    Modifier.padding(vertical = 8.dp).let { base -> item.link?.let { link -> base.clickable { open(link) } } ?: base },
    verticalArrangement = Arrangement.spacedBy(8.dp),
  ) {
    Box {
      Artwork(item.image, Modifier.fillMaxWidth().height(150.dp).clip(RoundedCornerShape(5.dp)), ContentScale.Crop)
      Box(Modifier.padding(10.dp).size(24.dp).clip(CircleShape).background(Color.White), contentAlignment = Alignment.Center) {
        Text("$index", color = Color.Black, fontSize = 12.sp, fontWeight = FontWeight.Bold)
      }
    }
    Text(item.title.uppercase(), fontSize = 18.sp, fontWeight = FontWeight.Bold)
  }
}

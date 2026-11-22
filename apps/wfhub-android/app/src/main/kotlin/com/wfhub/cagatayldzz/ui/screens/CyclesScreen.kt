package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.wfhub.cagatayldzz.data.Http
import com.wfhub.cagatayldzz.data.load
import com.wfhub.cagatayldzz.ui.*
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import uniffi.wfhub_core.CycleState
import uniffi.wfhub_core.Route
import uniffi.wfhub_core.activityUrl
import uniffi.wfhub_core.cycleRoutes
import uniffi.wfhub_core.parseCycle

@Composable
fun CyclesScreen(nav: NavController) {
  val settings = LocalSettings.current
  val state = load(settings.platform, refreshSeconds = 60) {
    coroutineScope {
      cycleRoutes().map { route ->
        async {
          val body = Http.getOrNull(activityUrl(settings.platform, route.endpoint))
          route to body?.let { runCatching { parseCycle(it) }.getOrNull() }
        }
      }.awaitAll()
    }
  }
  DetailScreen(nav, t("cycles")) {
    LoadedContent(state) { cycles ->
      LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        items(cycles) { (route, cycle) -> CycleCard(route, cycle) }
      }
    }
  }
}

@Composable
private fun CycleCard(route: Route, cycle: CycleState?) {
  WFCard {
    Text(td(route.title), fontWeight = FontWeight.Bold)
    if (cycle == null) {
      Text(t("unableToLoad"), color = MaterialTheme.colorScheme.onSurfaceVariant)
    } else {
      Row {
        Text(td(cycle.state), color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f).padding(end = 8.dp))
        if (isFuture(cycle.expiry)) Countdown(cycle.expiry) else if (cycle.expiry != null) Text(t("expired"))
      }
    }
  }
}

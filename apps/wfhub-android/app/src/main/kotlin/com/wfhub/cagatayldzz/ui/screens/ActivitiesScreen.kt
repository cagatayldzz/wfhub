package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.wfhub.cagatayldzz.ui.Artwork
import com.wfhub.cagatayldzz.ui.Routes
import com.wfhub.cagatayldzz.ui.ScreenTitle
import com.wfhub.cagatayldzz.ui.t
import com.wfhub.cagatayldzz.ui.td
import uniffi.wfhub_core.activityImageUrl
import uniffi.wfhub_core.activityRoutes

@Composable
fun ActivitiesScreen(nav: NavController) {
  val routes = activityRoutes()
  LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
    item { ScreenTitle(t("activities"), t("worldstateDetails")) }
    items(routes) { route ->
      Row(
        Modifier.fillMaxWidth().defaultMinSize(minHeight = 58.dp).clip(RoundedCornerShape(12.dp))
          .clickable { nav.navigate(if (route.endpoint == "cycles") "cycles" else Routes.activity(route.endpoint)) }
          .padding(horizontal = 14.dp, vertical = 9.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
      ) {
        Artwork(activityImageUrl(route.endpoint), Modifier.size(40.dp))
        Text(td(route.title), fontWeight = androidx.compose.ui.text.font.FontWeight.SemiBold, modifier = Modifier.weight(1f))
        Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, contentDescription = null, tint = MaterialTheme.colorScheme.onSurfaceVariant)
      }
    }
  }
}

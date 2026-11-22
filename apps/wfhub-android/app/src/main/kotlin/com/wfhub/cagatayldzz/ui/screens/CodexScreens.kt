package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.MenuBook
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.wfhub.cagatayldzz.data.Http
import com.wfhub.cagatayldzz.data.load
import com.wfhub.cagatayldzz.ui.*
import uniffi.wfhub_core.CodexItem
import uniffi.wfhub_core.DetailLine
import uniffi.wfhub_core.categoryImageUrl
import uniffi.wfhub_core.categoryRoutes
import uniffi.wfhub_core.categoryUrl
import uniffi.wfhub_core.filterTiles
import uniffi.wfhub_core.itemUrl
import uniffi.wfhub_core.parseCategory
import uniffi.wfhub_core.parseItem

/** The Codex tab. (The Mastery Rank checklist of the iOS app is not part of the Android app yet.) */
@Composable
fun CodexScreen(nav: NavController) {
  Column(Modifier.padding(16.dp)) {
    ScreenTitle(t("codex"))
    WFCard(onClick = { nav.navigate("catalogue") }) {
      Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
        Icon(Icons.AutoMirrored.Outlined.MenuBook, contentDescription = null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(30.dp))
        Column {
          Text(t("codexCatalogue"), fontWeight = FontWeight.Bold)
          Text(t("browseCategories"), color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
      }
    }
  }
}

@Composable
fun CatalogueScreen(nav: NavController) {
  DetailScreen(nav, t("codexCatalogue")) {
    LazyVerticalGrid(GridCells.Fixed(2), contentPadding = PaddingValues(10.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
      items(categoryRoutes()) { route -> ArtworkCard(route.title, categoryImageUrl(route.endpoint)) { nav.navigate(Routes.category(route.endpoint)) } }
    }
  }
}

@Composable
fun CategoryScreen(nav: NavController, category: String) {
  val state = load(category) { parseCategory(Http.get(categoryUrl(category))) }
  var query by rememberSaveable { mutableStateOf("") }
  DetailScreen(nav, td(categoryRoutes().firstOrNull { it.endpoint == category }?.title ?: category)) {
    LoadedContent(state) { tiles ->
      val shown = filterTiles(tiles, query)
      LazyVerticalGrid(GridCells.Fixed(2), contentPadding = PaddingValues(12.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item(span = { GridItemSpan(2) }) { OutlinedTextField(query, { query = it }, Modifier.fillMaxWidth(), singleLine = true, placeholder = { Text(t("search")) }) }
        if (shown.isEmpty()) item(span = { GridItemSpan(2) }) { Text(t("noDataFound"), color = MaterialTheme.colorScheme.onSurfaceVariant) }
        items(shown) { tile -> ArtworkCard(tile.title, tile.image) { nav.navigate(Routes.item(category, tile.slug)) } }
      }
    }
  }
}

@Composable
fun ItemScreen(nav: NavController, category: String, slug: String) {
  val state = load(category to slug) { parseItem(Http.get(itemUrl(category, slug))) }
  DetailScreen(nav, state.value?.title ?: "") { LoadedContent(state) { ItemView(it) } }
}

@Composable
private fun ItemView(item: CodexItem) {
  LazyColumn(contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
    item {
      Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(WF.artwork).padding(20.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Artwork(item.image, Modifier.height(190.dp).fillMaxWidth())
        Text(item.title, color = Color.White, fontSize = 22.sp, fontWeight = FontWeight.Bold)
      }
    }
    item.description?.let { description ->
      item { WFCard { Text(t("description"), fontWeight = FontWeight.Bold); Text(description, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
    }
    if (item.stats.isNotEmpty()) item { WFCard { Text(t("information"), fontWeight = FontWeight.Bold); item.stats.forEach { Line(it) } } }
    if (item.abilities.isNotEmpty()) {
      item {
        WFCard {
          Text(t("abilities"), fontWeight = FontWeight.Bold)
          item.abilities.forEach { ability ->
            Row(Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
              Artwork(ability.image, Modifier.size(48.dp).clip(RoundedCornerShape(7.dp)).background(WF.artwork))
              Column {
                Text(ability.name, fontWeight = FontWeight.Bold)
                ability.description?.let { Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant) }
              }
            }
          }
        }
      }
    }
    if (item.damage.isNotEmpty()) item { WFCard { Text(t("damage"), fontWeight = FontWeight.Bold); item.damage.forEach { Line(it) } } }
  }
}

/** A `label: value` row (or a header when there is no value), indented by its depth. */
@Composable
private fun Line(line: DetailLine) {
  Row(Modifier.fillMaxWidth().padding(start = (line.depth.toInt() * 12).dp)) {
    Text(td(line.label), fontWeight = if (line.value == null) FontWeight.Bold else FontWeight.Normal, color = if (line.value == null) Color.Unspecified else MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(1f))
    line.value?.let { Text(it, fontWeight = FontWeight.SemiBold) }
  }
}

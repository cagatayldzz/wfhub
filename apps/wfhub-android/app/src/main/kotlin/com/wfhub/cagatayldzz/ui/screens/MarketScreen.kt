package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.wfhub.cagatayldzz.data.Http
import com.wfhub.cagatayldzz.data.load
import com.wfhub.cagatayldzz.ui.*
import uniffi.wfhub_core.MarketBook
import uniffi.wfhub_core.OrderRow
import uniffi.wfhub_core.marketItemsUrl
import uniffi.wfhub_core.marketOrdersUrl

@Composable
fun MarketScreen() {
  // The book keeps the item catalogue, so it is only downloaded again once an hour.
  val book = remember { MarketBook() }
  var type by rememberSaveable { mutableStateOf("sell") }
  val state = load(Unit, refreshSeconds = 60) {
    val now = System.currentTimeMillis() / 1000
    val orders = Http.get(marketOrdersUrl())
    if (book.needsCatalogue(now)) book.setCatalogue(Http.get(marketItemsUrl()), now)
    book.rows(orders)
  }
  val uri = LocalUriHandler.current
  Column(Modifier.padding(horizontal = 16.dp)) {
    ScreenTitle(t("market"))
    SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth().padding(bottom = 8.dp)) {
      listOf("sell", "buy").forEachIndexed { index, kind ->
        SegmentedButton(selected = type == kind, onClick = { type = kind }, shape = SegmentedButtonDefaults.itemShape(index, 2)) { Text(t(kind)) }
      }
    }
    LoadedContent(state) { rows ->
      LazyColumn(contentPadding = PaddingValues(bottom = 16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        items(if (type == "sell") rows.sell else rows.buy) { row -> OrderCard(row) { link -> uri.openUri(link) } }
      }
    }
  }
}

@Composable
private fun OrderCard(row: OrderRow, open: (String) -> Unit) {
  WFCard(onClick = row.link?.let { link -> { open(link) } }) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
      Artwork(row.icon, Modifier.size(76.dp).clip(RoundedCornerShape(10.dp)).background(WF.artwork).padding(8.dp))
      Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(row.name, fontWeight = FontWeight.Bold, maxLines = 2)
        Text(row.user, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
        Text("×${row.quantity}", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
      Column(horizontalAlignment = Alignment.End) {
        Text("${row.platinum}", fontSize = 17.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.primary)
        Text(t("platinum"), fontSize = 9.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant)
      }
    }
  }
}

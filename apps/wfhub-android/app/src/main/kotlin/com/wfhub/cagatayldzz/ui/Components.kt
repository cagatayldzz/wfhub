package com.wfhub.cagatayldzz.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import com.wfhub.cagatayldzz.data.Loaded
import kotlinx.coroutines.delay
import uniffi.wfhub_core.countdown
import uniffi.wfhub_core.expiryUnix

@Composable
fun WFCard(modifier: Modifier = Modifier, onClick: (() -> Unit)? = null, content: @Composable () -> Unit) {
  val shape = RoundedCornerShape(12.dp)
  Column(
    modifier
      .fillMaxWidth()
      .clip(shape)
      .background(MaterialTheme.colorScheme.surfaceVariant)
      .border(1.dp, MaterialTheme.colorScheme.onSurface.copy(alpha = 0.07f), shape)
      .let { if (onClick != null) it.clickable(onClick = onClick) else it }
      .padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(6.dp),
  ) { content() }
}

@Composable
fun ScreenTitle(text: String, subtitle: String? = null) {
  Column(Modifier.padding(bottom = 12.dp)) {
    Text(text, fontSize = 28.sp, fontWeight = FontWeight.ExtraBold)
    if (subtitle != null) Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
  }
}

@Composable
fun SectionHeader(title: String, subtitle: String, onViewAll: (() -> Unit)? = null) {
  Row(Modifier.fillMaxWidth().padding(top = 24.dp, bottom = 12.dp), verticalAlignment = Alignment.Bottom) {
    Column(Modifier.weight(1f)) {
      Text(title, style = MaterialTheme.typography.titleLarge, fontWeight = FontWeight.Bold)
      Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
    if (onViewAll != null) Text(t("viewAll"), color = MaterialTheme.colorScheme.primary, modifier = Modifier.clickable(onClick = onViewAll).padding(8.dp))
  }
}

/** "2d 3h 4m" until `expiry`, refreshed every 30 seconds. The date maths is the Rust core's. */
@Composable
fun Countdown(expiry: String?, modifier: Modifier = Modifier, color: Color = Color.Unspecified) {
  if (expiry == null) return
  var now by remember { mutableLongStateOf(System.currentTimeMillis() / 1000) }
  LaunchedEffect(Unit) {
    while (true) {
      delay(30_000)
      now = System.currentTimeMillis() / 1000
    }
  }
  countdown(expiry, now)?.let { Text(it, fontWeight = FontWeight.Bold, modifier = modifier, color = color) }
}

/** Whether `expiry` is still ahead; lists show a countdown only then. */
fun isFuture(expiry: String?): Boolean = expiry?.let { expiryUnix(it) }?.let { it > System.currentTimeMillis() / 1000 } ?: false

@Composable
fun Artwork(url: String?, modifier: Modifier = Modifier, contentScale: ContentScale = ContentScale.Fit) {
  if (url == null) Box(modifier) else AsyncImage(model = url, contentDescription = null, modifier = modifier, contentScale = contentScale)
}

/** A dark tile with artwork and a title (Codex, categories). */
@Composable
fun ArtworkCard(title: String, image: String?, onClick: () -> Unit) {
  Column(
    Modifier.clip(RoundedCornerShape(10.dp)).background(WF.artwork).clickable(onClick = onClick).padding(10.dp).fillMaxWidth(),
    horizontalAlignment = Alignment.CenterHorizontally,
    verticalArrangement = Arrangement.spacedBy(6.dp),
  ) {
    Artwork(image, Modifier.height(100.dp).fillMaxWidth())
    Text(td(title), color = Color.White, fontWeight = FontWeight.Bold, fontSize = 15.sp, textAlign = TextAlign.Center, maxLines = 2, modifier = Modifier.height(40.dp))
  }
}

@Composable
fun ErrorView(message: String, onRetry: () -> Unit) {
  Column(Modifier.fillMaxSize().padding(32.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
    Text(t("unableToLoad"), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
    Text(message, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, modifier = Modifier.padding(vertical = 8.dp))
    Button(onClick = onRetry) { Text(t("retry")) }
  }
}

@Composable
fun Loading() {
  Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { CircularProgressIndicator(Modifier.size(32.dp)) }
}

/** Loading, error or the content, with pull to refresh. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun <T> LoadedContent(state: Loaded<T>, modifier: Modifier = Modifier, content: @Composable (T) -> Unit) {
  PullToRefreshBox(isRefreshing = state.loading && state.value != null, onRefresh = state.reload, modifier = modifier.fillMaxSize()) {
    val value = state.value
    when {
      value != null -> content(value)
      state.error != null -> ErrorView(state.error, state.reload)
      else -> Loading()
    }
  }
}

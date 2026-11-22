package com.wfhub.cagatayldzz.data

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay

/** What a screen shows: the last good `value`, whether a load is running and the `error` of the last failed one. */
class Loaded<T>(val value: T?, val loading: Boolean, val error: String?, val reload: () -> Unit)

/**
 * Runs `block` when `key` changes (and on `reload`), then again every `refreshSeconds` (0 = once), like the 60 second refresh of the
 * iOS screens. A failed refresh keeps the previous value.
 */
@Composable
fun <T> load(key: Any?, refreshSeconds: Long = 0, block: suspend () -> T): Loaded<T> {
  var value by remember(key) { mutableStateOf<T?>(null) }
  var error by remember(key) { mutableStateOf<String?>(null) }
  var loading by remember(key) { mutableStateOf(true) }
  var tick by remember(key) { mutableIntStateOf(0) }
  LaunchedEffect(key, tick) {
    while (true) {
      try {
        value = block()
        error = null
      } catch (e: CancellationException) {
        throw e
      } catch (e: Exception) {
        error = e.message ?: e.javaClass.simpleName
      }
      loading = false
      if (refreshSeconds <= 0) break
      delay(refreshSeconds * 1000)
    }
  }
  return Loaded(value, loading, error) {
    loading = true
    tick++
  }
}

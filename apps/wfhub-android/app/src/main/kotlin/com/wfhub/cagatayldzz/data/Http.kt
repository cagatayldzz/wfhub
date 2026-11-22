package com.wfhub.cagatayldzz.data

import java.net.HttpURLConnection
import java.net.URL
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** The transport. What the answers mean is the business of the Rust core (`uniffi.wfhub_core`). */
object Http {
  private const val TIMEOUT_MS = 30_000

  class StatusException(val status: Int) : Exception("HTTP $status")

  suspend fun get(url: String): String = withContext(Dispatchers.IO) {
    val connection = URL(url).openConnection() as HttpURLConnection
    try {
      connection.connectTimeout = TIMEOUT_MS
      connection.readTimeout = TIMEOUT_MS
      connection.setRequestProperty("Accept", "application/json")
      val status = connection.responseCode
      if (status !in 200..299) throw StatusException(status)
      connection.inputStream.bufferedReader().use { it.readText() }
    } finally {
      connection.disconnect()
    }
  }

  /** `get`, but a failure is `null` (one failed request must not fail a whole screen). */
  suspend fun getOrNull(url: String): String? =
    try {
      get(url)
    } catch (e: CancellationException) {
      throw e
    } catch (e: Exception) {
      null
    }

  /** POSTs JSON and returns the HTTP status. */
  suspend fun post(url: String, body: String): Int = withContext(Dispatchers.IO) {
    val connection = URL(url).openConnection() as HttpURLConnection
    try {
      connection.requestMethod = "POST"
      connection.connectTimeout = TIMEOUT_MS
      connection.readTimeout = TIMEOUT_MS
      connection.doOutput = true
      connection.setRequestProperty("Content-Type", "application/json")
      connection.outputStream.use { it.write(body.toByteArray()) }
      connection.responseCode
    } finally {
      connection.disconnect()
    }
  }
}

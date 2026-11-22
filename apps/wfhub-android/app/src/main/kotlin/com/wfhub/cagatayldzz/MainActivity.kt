package com.wfhub.cagatayldzz

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import com.wfhub.cagatayldzz.data.Settings
import com.wfhub.cagatayldzz.ui.WFHubApp

class MainActivity : ComponentActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    enableEdgeToEdge()
    val settings = Settings(applicationContext)
    setContent { WFHubApp(settings) }
  }
}

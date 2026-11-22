package com.wfhub.cagatayldzz.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavController

/** A screen below a tab: title and back button. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DetailScreen(nav: NavController, title: String, content: @Composable () -> Unit) {
  Column(Modifier.fillMaxSize()) {
    TopAppBar(
      title = { Text(title, maxLines = 1) },
      navigationIcon = { IconButton(onClick = { nav.popBackStack() }) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = null) } },
    )
    content()
  }
}

package com.wfhub.cagatayldzz.ui

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.MenuBook
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.ShoppingBag
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.navigation.NavController
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.NavHostController
import androidx.navigation.navArgument
import com.wfhub.cagatayldzz.data.Settings
import com.wfhub.cagatayldzz.ui.screens.ActivitiesScreen
import com.wfhub.cagatayldzz.ui.screens.ActivityScreen
import com.wfhub.cagatayldzz.ui.screens.CatalogueScreen
import com.wfhub.cagatayldzz.ui.screens.CategoryScreen
import com.wfhub.cagatayldzz.ui.screens.CodexScreen
import com.wfhub.cagatayldzz.ui.screens.CyclesScreen
import com.wfhub.cagatayldzz.ui.screens.FeedbackScreen
import com.wfhub.cagatayldzz.ui.screens.HomeScreen
import com.wfhub.cagatayldzz.ui.screens.ItemScreen
import com.wfhub.cagatayldzz.ui.screens.MarketScreen
import com.wfhub.cagatayldzz.ui.screens.SettingsScreen

private class Tab(val route: String, val titleKey: String, val icon: ImageVector)

private val tabs = listOf(
  Tab("home", "home", Icons.Outlined.Home),
  Tab("activities", "activities", Icons.Outlined.Bolt),
  Tab("market", "market", Icons.Outlined.ShoppingBag),
  Tab("codex", "codex", Icons.AutoMirrored.Outlined.MenuBook),
  Tab("settings", "settings", Icons.Outlined.Settings),
)

/** Screens of the app; arguments are part of the path. */
object Routes {
  fun activity(endpoint: String) = "activity/$endpoint"
  fun category(category: String) = "category/$category"
  fun item(category: String, slug: String) = "item/$category/$slug"
}

@Composable
fun WFHubApp(settings: Settings) {
  WFHubTheme(settings) {
    val nav = rememberNavController()
    val route by nav.currentBackStackEntryAsState()
    val current = route?.destination?.route
    Scaffold(
      bottomBar = {
        // The tab bar shows on the five main screens; detail screens bring their own back button.
        if (tabs.any { it.route == current }) {
          NavigationBar {
            tabs.forEach { tab ->
              NavigationBarItem(
                selected = tab.route == current,
                onClick = { nav.openTab(tab.route) },
                icon = { Icon(tab.icon, contentDescription = null) },
                label = { Text(t(tab.titleKey)) },
              )
            }
          }
        }
      },
    ) { padding -> AppNavHost(nav, padding) }
  }
}

private fun NavController.openTab(route: String) {
  navigate(route) {
    popUpTo(graph.findStartDestination().id) { saveState = true }
    launchSingleTop = true
    restoreState = true
  }
}

@Composable
private fun AppNavHost(nav: NavHostController, padding: PaddingValues) {
  val text = NavType.StringType
  NavHost(nav, startDestination = "home", modifier = Modifier.padding(padding)) {
    composable("home") { HomeScreen(nav) }
    composable("activities") { ActivitiesScreen(nav) }
    composable("market") { MarketScreen() }
    composable("codex") { CodexScreen(nav) }
    composable("settings") { SettingsScreen(nav) }
    composable("catalogue") { CatalogueScreen(nav) }
    composable("cycles") { CyclesScreen(nav) }
    composable("feedback") { FeedbackScreen(nav) }
    composable("activity/{endpoint}", listOf(navArgument("endpoint") { type = text })) {
      ActivityScreen(nav, it.arguments?.getString("endpoint").orEmpty())
    }
    composable("category/{category}", listOf(navArgument("category") { type = text })) {
      CategoryScreen(nav, it.arguments?.getString("category").orEmpty())
    }
    composable("item/{category}/{slug}", listOf(navArgument("category") { type = text }, navArgument("slug") { type = text })) {
      ItemScreen(nav, it.arguments?.getString("category").orEmpty(), it.arguments?.getString("slug").orEmpty())
    }
  }
}

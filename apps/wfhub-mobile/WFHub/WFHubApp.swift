import SwiftUI

@main
struct WFHubApp: App {
  @StateObject private var ads = AdsManager()
  var body: some Scene {
    WindowGroup { RootView().environmentObject(ads).task { await ads.prepare() } }
  }
}

struct Route: Identifiable, Hashable {
  let title: String
  let endpoint: String
  var id: String { endpoint }
}

enum Catalogue {
  static let routes: (activities: [Route], categories: [Route]) = (
    activities: [
      Route(title: "Alerts", endpoint: "alerts"),
      Route(title: "Baro Ki'Teer", endpoint: "voidTrader"),
      Route(title: "Fissures", endpoint: "fissures"),
      Route(title: "Nightwave", endpoint: "nightwave"),
      Route(title: "Events", endpoint: "events"),
      Route(title: "Invasions", endpoint: "invasions"),
      Route(title: "Arbitration", endpoint: "arbitration"),
      Route(title: "Archon Hunt", endpoint: "archonHunt"),
      Route(title: "Steel Path", endpoint: "steelPath"),
      Route(title: "Sentient Outposts", endpoint: "sentientOutposts"),
      Route(title: "Simaris", endpoint: "simaris"),
      Route(title: "Construction Progress", endpoint: "constructionProgress"),
      Route(title: "News", endpoint: "news"),
      Route(title: "Market", endpoint: "flashSales"),
      Route(title: "Sortie", endpoint: "sortie"),
      Route(title: "Daily Deals", endpoint: "dailyDeals"),
      Route(title: "Earth Cycle", endpoint: "earthCycle"),
      Route(title: "Cetus Cycle", endpoint: "cetusCycle"),
      Route(title: "Vallis Cycle", endpoint: "vallisCycle"),
      Route(title: "Cambion Cycle", endpoint: "cambionCycle"),
      Route(title: "Duviri Cycle", endpoint: "duviriCycle"),
      Route(title: "Zariman Cycle", endpoint: "zarimanCycle"),
    ],
    categories: [
      "arcanes",
      "arch-gun",
      "arch-melee",
      "archwing",
      "enemy",
      "fish",
      "gear",
      "glyphs",
      "melee",
      "misc",
      "mods",
      "node",
      "pets",
      "primary",
      "quests",
      "railjack",
      "relics",
      "resources",
      "secondary",
      "sentinels",
      "sentinelweapons",
      "sigils",
      "skins",
      "warframes",
    ].map { Route(title: $0.replacingOccurrences(of: "-", with: " ").capitalized, endpoint: $0) }
  )
}

struct RootView: View {
  @State private var selection =
    ProcessInfo.processInfo.arguments.firstIndex(of: "--preview-tab").flatMap { index in
      let args = ProcessInfo.processInfo.arguments
      return index + 1 < args.count ? Int(args[index + 1]) : nil
    } ?? 0
  @AppStorage("appLanguage") private var language = "system"
  @AppStorage("appTheme") private var theme = "system"
  private var colorScheme: ColorScheme? {
    switch theme {
    case "light": return .light
    case "dark": return .dark
    default: return nil
    }
  }
  var body: some View {
    TabView(selection: $selection) {
      NavigationStack { HomeView() }.tabItem { Label(t("home"), systemImage: "house") }.tag(0)
      NavigationStack { ActivitiesView() }.tabItem { Label(t("activities"), systemImage: "bolt") }.tag(1)
      NavigationStack { MarketView() }.tabItem { Label(t("market"), systemImage: "bag") }.tag(2)
      NavigationStack { CodexCategoriesView() }.tabItem { Label(t("codex"), systemImage: "book") }.tag(3)
      NavigationStack { SettingsView() }.tabItem { Label(t("settings"), systemImage: "gear") }.tag(4)
    }
    .id(language)
    .tint(WFStyle.accent)
    .preferredColorScheme(colorScheme)
    .environment(\.locale, language == "system" ? .autoupdatingCurrent : Locale(identifier: language))
  }
}

enum RouteKind { case activity, category }

import SwiftUI

@main
struct WFHubApp: App {
  @StateObject private var ads = AdsManager()
  var body: some Scene {
    WindowGroup { RootView().environmentObject(ads).task { await ads.prepare() } }
  }
}

// `Route` (title and endpoint of a screen) and the lists of routes come from the Rust core.
extension Route: Identifiable {
  public var id: String { endpoint }
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

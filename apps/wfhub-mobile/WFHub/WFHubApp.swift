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
  static let routes: (activities: [Route], categories: [Route]) = {
    guard let pointer = wfhub_catalogue() else { return ([], []) }
    defer { wfhub_free(pointer) }
    let lines = String(cString: pointer).split(separator: "\n")
    var activities: [Route] = []
    var categories: [Route] = []
    for line in lines {
      let fields = line.split(separator: "\t").map(String.init)
      if fields.count == 3 && fields[0] == "activity" {
        activities.append(Route(title: fields[1], endpoint: fields[2]))
      } else if fields.count == 2 && fields[0] == "category" {
        categories.append(Route(title: fields[1].replacingOccurrences(of: "-", with: " ").capitalized, endpoint: fields[1]))
      }
    }
    return (activities, categories)
  }()
}

struct RootView: View {
  @State private var selection = ProcessInfo.processInfo.arguments.firstIndex(of: "--preview-tab").flatMap { index in
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
      NavigationStack { HomeView() }.tabItem { Label("Home", systemImage: "house") }.tag(0)
      NavigationStack { ActivitiesView() }.tabItem { Label("Activities", systemImage: "bolt") }.tag(1)
      NavigationStack { MarketView() }.tabItem { Label("Market", systemImage: "bag") }.tag(2)
      NavigationStack { CodexCategoriesView() }.tabItem { Label("Codex", systemImage: "book") }.tag(3)
      NavigationStack { SettingsView() }.tabItem { Label("Settings", systemImage: "gear") }.tag(4)
    }
    .tint(WFStyle.accent)
    .preferredColorScheme(colorScheme)
    .environment(\.locale, language == "system" ? .autoupdatingCurrent : Locale(identifier: language))
  }
}

enum RouteKind { case activity, category }

struct RouteList: View {
  let title: String
  let routes: [Route]
  let kind: RouteKind
  @State private var search = ""
  var body: some View {
    List(routes.filter { search.isEmpty || $0.title.localizedCaseInsensitiveContains(search) }) { route in
      NavigationLink(route.title) { DataScreen(route: route, kind: kind) }
    }
    .navigationTitle(title)
    .searchable(text: $search)
  }
}


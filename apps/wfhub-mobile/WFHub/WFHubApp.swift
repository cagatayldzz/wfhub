import SwiftUI

@main
struct WFHubApp: App {
  var body: some Scene {
    WindowGroup { RootView() }
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
  var body: some View {
    TabView {
      NavigationStack { HomeView() }.tabItem { Label("Home", systemImage: "house") }
      NavigationStack { RouteList(title: "Activities", routes: Catalogue.routes.activities, kind: .activity) }.tabItem { Label("Activities", systemImage: "bolt") }
      NavigationStack { RouteList(title: "Codex", routes: Catalogue.routes.categories, kind: .category) }.tabItem { Label("Codex", systemImage: "book") }
      NavigationStack { MarketView() }.tabItem { Label("Market", systemImage: "cart") }
      NavigationStack { SettingsView() }.tabItem { Label("Settings", systemImage: "gear") }
    }
    .tint(.orange)
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

struct SettingsView: View {
  @AppStorage("tennoName") private var name = "Tenno"
  @AppStorage("platform") private var platform = "pc"
  var body: some View {
    Form {
      TextField("Tenno name", text: $name)
      Picker("Platform", selection: $platform) {
        Text("PC").tag("pc")
        Text("PlayStation").tag("ps4")
        Text("Xbox").tag("xb1")
        Text("Nintendo Switch").tag("swi")
      }
    }.navigationTitle("Settings")
  }
}

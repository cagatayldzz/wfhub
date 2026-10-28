import SwiftUI

struct ActivitiesView: View {
  private var routes: [Route] {
    var result = Catalogue.routes.activities.filter { !$0.endpoint.hasSuffix("Cycle") }
    if let index = result.firstIndex(where: { $0.endpoint == "fissures" }) {
      result.insert(Route(title: "Cycles", endpoint: "cycles"), at: index + 1)
    }
    return result
  }

  var body: some View {
    ScrollView {
      LazyVStack(spacing: 0) {
        ForEach(routes) { route in
          if route.endpoint == "cycles" {
            NavigationLink { CyclesView() } label: { ActivityLink(route: route) }
          } else {
            NavigationLink { DataScreen(route: route, kind: .activity) } label: { ActivityLink(route: route) }
          }
          Divider().padding(.leading, 76)
        }
      }.padding(.top, 8)
    }.navigationTitle("Activities")
  }
}

private struct ActivityLink: View {
  let route: Route
  private var imageName: String {
    switch route.endpoint {
    case "voidTrader": return "baro-ki-teer"
    case "dailyDeals": return "void-trader"
    case "archonHunt": return "archon-hunt"
    case "steelPath": return "steel-path"
    case "sentientOutposts": return "sentient-outpost"
    case "constructionProgress": return "construction-progress"
    case "flashSales": return "market"
    default: return route.endpoint
    }
  }
  var body: some View {
    HStack(spacing: 14) {
      AsyncImage(url: URL(string: "https://cdn.cagatayldzz.com/constants/wfhub/activities/\(imageName).png")) { image in
        image.resizable().scaledToFill()
      } placeholder: { Color.secondary.opacity(0.12) }
      .frame(width: 44, height: 44).clipped().cornerRadius(6)
      Text(route.title).font(.system(size: 18, weight: .bold)).foregroundStyle(.primary)
      Spacer()
      Image(systemName: "chevron.right").font(.caption.bold()).foregroundStyle(.tertiary)
    }.padding(.horizontal, 20).padding(.vertical, 12).contentShape(Rectangle())
  }
}

struct CyclesView: View {
  private let routes = [
    Route(title: "Plains of Eidolon", endpoint: "cetusCycle"),
    Route(title: "Orb Vallis", endpoint: "vallisCycle"),
    Route(title: "Cambion Drift", endpoint: "cambionCycle"),
    Route(title: "Zariman Ten Zero", endpoint: "zarimanCycle"),
    Route(title: "Earth Forests", endpoint: "earthCycle"),
    Route(title: "Duviri", endpoint: "duviriCycle")
  ]
  @AppStorage("platform") private var platform = "pc"
  @State private var cycles: [String: [String: Any]] = [:]
  @State private var failed: Set<String> = []
  @State private var loading = true

  var body: some View {
    ScrollView {
      LazyVStack(spacing: 10) {
      ForEach(routes) { route in
        WFCard { VStack(alignment: .leading, spacing: 8) {
          Text(route.title).font(.headline)
          if let cycle = cycles[route.endpoint] {
            HStack {
              Text((cycle["state"] as? String ?? "–").capitalized).foregroundStyle(.secondary)
              Spacer()
              if let expiry = cycle["expiry"] as? String, let date = screenDate(expiry) {
                if date > .now { Text(date, style: .timer).monospacedDigit() } else { Text("Expired") }
              }
            }
          } else if failed.contains(route.endpoint) { Text("Unable to load").foregroundStyle(.secondary) }
          else if loading { ProgressView() }
        } }
      }
      if !failed.isEmpty { Button("Retry") { Task { await load() } } }
      }.padding(16)
    }.navigationTitle("Cycles").refreshable { await load() }.task(id: platform) {
      await load()
      while !Task.isCancelled {
        do { try await Task.sleep(nanoseconds: 60_000_000_000) } catch { return }
        await load()
      }
    }
  }
  @MainActor private func load() async {
    let selected = platform
    var results: [String: [String: Any]] = [:]
    var errors: Set<String> = []
    for route in routes {
      guard !Task.isCancelled, let url = URL(string: "https://api.warframestat.us/\(selected)/\(route.endpoint)/") else { return }
      if let value = try? await API.fetch(url) as? [String: Any] { results[route.endpoint] = value }
      else { errors.insert(route.endpoint) }
    }
    guard !Task.isCancelled, platform == selected else { return }
    cycles.merge(results) { _, new in new }
    failed = errors
    loading = false
  }
}

struct SettingsView: View {
  @AppStorage("tennoName") private var storedName = "Tenno"
  @AppStorage("platform") private var storedPlatform = "pc"
  @State private var name = ""
  @State private var platform = "pc"
  @State private var saved = false
  @AppStorage("appLanguage") private var language = "system"
  @AppStorage("appTheme") private var theme = "system"

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 22) {
        VStack(alignment: .leading, spacing: 8) {
          Text("Tenno name").font(.subheadline.bold()).foregroundStyle(.secondary)
          TextField("Tenno name", text: $name).textInputAutocapitalization(.never)
            .padding(14).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 10))
        }
        VStack(alignment: .leading, spacing: 8) {
          Text("Platform").font(.subheadline.bold()).foregroundStyle(.secondary)
          Picker("Platform", selection: $platform) {
            Text("PC").tag("pc")
            Text("Xbox").tag("xb1")
            Text("Nintendo Switch").tag("swi")
            Text("PlayStation").tag("ps4")
          }.pickerStyle(.menu).frame(maxWidth: .infinity, alignment: .leading)
            .padding(10).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 10))
        }
        VStack(alignment: .leading, spacing: 8) {
          Text("Language").font(.subheadline.bold()).foregroundStyle(.secondary)
          Picker("Language", selection: $language) {
            Text("System").tag("system")
            Text("Deutsch").tag("de")
            Text("English").tag("en")
            Text("Español").tag("es")
            Text("Français").tag("fr")
            Text("Italiano").tag("it")
            Text("日本語").tag("ja")
            Text("한국어").tag("ko")
            Text("Polski").tag("pl")
            Text("Português").tag("pt")
            Text("Русский").tag("ru")
            Text("ไทย").tag("th")
            Text("Türkçe").tag("tr")
            Text("Українська").tag("uk")
            Text("简体中文").tag("zh")
          }.pickerStyle(.menu).frame(maxWidth: .infinity, alignment: .leading)
            .padding(10).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 10))
        }
        VStack(alignment: .leading, spacing: 8) {
          Text("Appearance").font(.subheadline.bold()).foregroundStyle(.secondary)
          Picker("Appearance", selection: $theme) {
            Text("System").tag("system")
            Text("Light").tag("light")
            Text("Dark").tag("dark")
          }.pickerStyle(.menu).frame(maxWidth: .infinity, alignment: .leading)
            .padding(10).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 10))
        }
        Button {
          storedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
          storedPlatform = platform
          saved = true
        } label: {
          Text("Save").font(.headline).foregroundStyle(.black).frame(maxWidth: .infinity).padding(15).background(WFStyle.gold).clipShape(RoundedRectangle(cornerRadius: 10))
        }
      }.padding(20)
    }.navigationTitle("Settings")
      .onAppear { name = storedName; platform = storedPlatform }
      .alert("Settings saved", isPresented: $saved) { Button("OK", role: .cancel) {} }
  }
}

struct CodexCategoriesView: View {
  private let columns = [GridItem(.flexible()), GridItem(.flexible())]
  var body: some View {
    ScrollView {
      LazyVGrid(columns: columns, spacing: 10) {
        ForEach(Catalogue.routes.categories.sorted { $0.title.localizedCaseInsensitiveCompare($1.title) == .orderedAscending }) { route in
          NavigationLink {
            DataScreen(route: route, kind: .category)
          } label: {
            ArtworkCard(title: route.title, image: URL(string: "https://cdn.cagatayldzz.com/constants/wfhub/codex/\(route.endpoint).png"))
          }.buttonStyle(.plain)
        }
      }.padding(10)
    }.navigationTitle("Codex")
  }
}

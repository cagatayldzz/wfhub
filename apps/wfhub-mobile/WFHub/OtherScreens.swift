import SwiftUI

struct ActivitiesView: View {
  private let order = ["alerts", "voidTrader", "fissures", "cycles", "dailyDeals", "nightwave", "events", "invasions", "arbitration", "archonHunt", "steelPath", "sentientOutposts", "simaris", "constructionProgress", "news", "flashSales", "sortie"]
  private var routes: [Route] {
    order.compactMap { endpoint in
      if endpoint == "cycles" { return Route(title: "Cycles", endpoint: endpoint) }
      return Catalogue.routes.activities.first { $0.endpoint == endpoint }
    }
  }

  var body: some View {
    ScrollView {
      LazyVStack(alignment: .leading, spacing: 8) {
        HStack(alignment: .firstTextBaseline) {
          VStack(alignment: .leading, spacing: 4) {
            Text("Activities").font(.system(size: 28, weight: .heavy))
            Text("Worldstate Details").font(.subheadline).foregroundStyle(.secondary)
          }
          Spacer()
        }.padding(.bottom, 10)
          ForEach(routes) { route in
            if route.endpoint == "cycles" {
              NavigationLink { CyclesView() } label: { ActivityLink(route: route) }.buttonStyle(.plain)
            } else {
              NavigationLink { DataScreen(route: route, kind: .activity) } label: { ActivityLink(route: route) }.buttonStyle(.plain)
            }
          }
      }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 24)
    }
    .navigationBarHidden(true)
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
    HStack(spacing: 12) {
      ThumbnailImage(url: URL(string: "https://cdn.cagatayldzz.com/constants/wfhub/activities/\(imageName).png"), maxPixelSize: 160)
      .frame(width: 40, height: 40)
      VStack(alignment: .leading, spacing: 3) {
        Text(LocalizedStringKey(route.title)).font(.system(size: 16, weight: .semibold)).foregroundStyle(.primary)
      }
      Spacer(minLength: 8)
      Image(systemName: "chevron.right").font(.system(size: 12, weight: .bold)).foregroundStyle(.secondary)
    }
    .padding(.horizontal, 14).padding(.vertical, 9)
    .frame(minHeight: 58)
    .frame(maxWidth: .infinity, alignment: .leading)
    .background(Color(uiColor: .secondarySystemBackground).opacity(0.65))
    .clipShape(RoundedRectangle(cornerRadius: 12))
    .contentShape(Rectangle())
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
  @EnvironmentObject private var ads: AdsManager
  @AppStorage("tennoName") private var storedName = "Tenno"
  @AppStorage("platform") private var storedPlatform = "pc"
  @AppStorage("appLanguage") private var language = "system"
  @AppStorage("appTheme") private var theme = "system"
  @State private var name = ""
  @State private var platform = "pc"
  @State private var saved = false
  @FocusState private var editingName: Bool

  private let languages: [(String, String)] = [
    ("de", "Deutsch"), ("en", "English"), ("es", "Español"), ("fr", "Français"),
    ("it", "Italiano"), ("ja", "日本語"), ("ko", "한국어"), ("pl", "Polski"),
    ("pt", "Português"), ("ru", "Русский"), ("th", "ไทย"), ("tr", "Türkçe"),
    ("uk", "Українська"), ("zh", "简体中文")
  ]
  private let platforms: [(String, String)] = [("pc", "PC"), ("xb1", "Xbox"), ("swi", "Nintendo Switch"), ("ps4", "PlayStation")]

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 24) {
        Text("Settings").font(.system(size: 28, weight: .heavy))

        VStack(alignment: .leading, spacing: 10) {
          Text("PROFILE").font(.caption.bold()).tracking(1).foregroundStyle(.secondary).padding(.horizontal, 4)
          VStack(spacing: 0) {
            HStack(spacing: 12) {
              SettingsIcon(name: "person.crop.circle")
              VStack(alignment: .leading, spacing: 4) {
                Text("Tenno Name").font(.subheadline).foregroundStyle(.secondary)
                TextField("Tenno Name", text: $name).textInputAutocapitalization(.never).autocorrectionDisabled()
                  .font(.body.weight(.semibold)).focused($editingName)
              }
            }.padding(16)
            Divider().padding(.leading, 60)
            Menu {
              ForEach(platforms, id: \.0) { code, title in
                Button { platform = code } label: { if platform == code { Label(title, systemImage: "checkmark") } else { Text(title) } }
              }
            } label: {
              SettingsRow(icon: "gamecontroller", title: "Platform", value: platforms.first { $0.0 == platform }?.1 ?? "PC")
            }
          }.tint(.primary).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
        }

        VStack(alignment: .leading, spacing: 10) {
          Text("PREFERENCES").font(.caption.bold()).tracking(1).foregroundStyle(.secondary).padding(.horizontal, 4)
          VStack(spacing: 0) {
            Menu {
              Button { language = "system" } label: { if language == "system" { Label("System", systemImage: "checkmark") } else { Text("System") } }
              ForEach(languages, id: \.0) { code, title in
                Button { language = code } label: { if language == code { Label(title, systemImage: "checkmark") } else { Text(title) } }
              }
            } label: {
              SettingsRow(icon: "globe", title: "Language", value: language == "system" ? "System" : languages.first { $0.0 == language }?.1 ?? "English")
            }
            Divider().padding(.leading, 60)
            Menu {
              Button { theme = "system" } label: { if theme == "system" { Label("System", systemImage: "checkmark") } else { Text("System") } }
              Button { theme = "light" } label: { if theme == "light" { Label("Light", systemImage: "checkmark") } else { Text("Light") } }
              Button { theme = "dark" } label: { if theme == "dark" { Label("Dark", systemImage: "checkmark") } else { Text("Dark") } }
            } label: {
              SettingsRow(icon: "circle.lefthalf.filled", title: "Appearance", value: theme == "light" ? "Light" : theme == "dark" ? "Dark" : "System")
            }
          }.tint(.primary).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
        }

        if ads.privacyOptionsRequired {
          Button { Task { await ads.showPrivacyOptions() } } label: {
            Label("Privacy options", systemImage: "hand.raised")
          }
        }

        Button {
          storedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
          storedPlatform = platform
          editingName = false
          saved = true
        } label: {
          Text("Save").font(.headline).foregroundStyle(.black).frame(maxWidth: .infinity).padding(.vertical, 16)
            .background(WFStyle.gold).clipShape(RoundedRectangle(cornerRadius: 12))
        }
      }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 32)
    }.scrollDismissesKeyboard(.interactively).navigationBarHidden(true)
      .onAppear { name = storedName; platform = storedPlatform }
      .alert("Settings saved", isPresented: $saved) { Button("OK", role: .cancel) {} }
  }
}

private struct SettingsIcon: View {
  let name: String
  var body: some View {
    Image(systemName: name).font(.system(size: 17, weight: .semibold)).foregroundStyle(WFStyle.accent)
      .frame(width: 34, height: 34).background(WFStyle.accent.opacity(0.1)).clipShape(RoundedRectangle(cornerRadius: 9))
  }
}

private struct SettingsRow: View {
  let icon: String
  let title: LocalizedStringKey
  let value: String
  var body: some View {
    HStack(spacing: 12) {
      SettingsIcon(name: icon)
      VStack(alignment: .leading, spacing: 4) {
        Text(title).font(.subheadline).foregroundStyle(Color(uiColor: .secondaryLabel))
        Text(LocalizedStringKey(value)).font(.body.weight(.semibold)).foregroundStyle(Color(uiColor: .label)).lineLimit(1)
      }
      Spacer(minLength: 8)
      Image(systemName: "chevron.up.chevron.down").font(.caption.weight(.semibold)).foregroundStyle(Color(uiColor: .secondaryLabel))
    }.padding(16).contentShape(Rectangle())
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

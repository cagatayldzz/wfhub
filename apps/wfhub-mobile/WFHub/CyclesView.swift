import SwiftUI

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

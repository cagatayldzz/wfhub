import SwiftUI

struct DataScreen: View {
  let route: Route
  let kind: RouteKind
  @AppStorage("platform") private var platform = "pc"
  @State private var data: Any?
  @State private var error: String?
  @State private var search = ""

  private var url: URL? {
    switch kind {
    case .activity: return URL(string: "https://api.warframestat.us/\(platform)/\(route.endpoint)/")
    case .category: return URL(string: "https://wfhub-api.cagatayldzz.com/api/\(route.endpoint).json")
    }
  }

  var body: some View {
    Group {
      if let error {
        ContentUnavailableView { Label("Unable to load", systemImage: "wifi.slash") } description: { Text(error) } actions: { Button("Retry") { Task { await load() } } }
      } else if let data {
        List {
          if let items = data as? [[String: Any]] {
            ForEach(Array(items.enumerated()), id: \.offset) { _, item in
              if matches(item) {
                if kind == .category, let slug = item["slug"] as? String {
                  NavigationLink(itemTitle(item)) { ItemScreen(category: route.endpoint, slug: slug, title: itemTitle(item)) }
                } else {
                  JsonRow(value: item)
                }
              }
            }
          } else if let object = data as? [String: Any] {
            JsonRow(value: object)
          } else {
            Text("No data available")
          }
        }.searchable(text: $search).refreshable { await load() }
      } else {
        ProgressView()
      }
    }
    .navigationTitle(route.title)
    .task(id: platform) { await load() }
  }

  private func matches(_ item: [String: Any]) -> Bool {
    search.isEmpty || itemTitle(item).localizedCaseInsensitiveContains(search)
  }

  private func load() async {
    guard let url else { error = "Invalid address"; return }
    do { data = try await API.fetch(url); error = nil } catch { self.error = error.localizedDescription }
  }
}

func itemTitle(_ item: [String: Any]) -> String {
  for key in ["name", "title", "node", "message", "description", "type", "id"] {
    if let value = item[key] as? String, !value.isEmpty { return value }
  }
  return "Details"
}

struct JsonRow: View {
  let value: [String: Any]
  var body: some View {
    VStack(alignment: .leading, spacing: 6) {
      Text(itemTitle(value)).font(.headline)
      ForEach(value.keys.sorted().filter { !["id", "name", "title"].contains($0) }, id: \.self) { key in
        if let text = value[key] as? String, !text.isEmpty, text.count < 300 {
          LabeledContent(key, value: text)
            .font(.caption).foregroundStyle(.secondary)
        } else if let number = value[key] as? NSNumber {
          LabeledContent(key, value: number.stringValue).font(.caption).foregroundStyle(.secondary)
        }
      }
    }.padding(.vertical, 4)
  }
}

enum API {
  static func fetch(_ url: URL) async throws -> Any {
    let (data, response) = try await URLSession.shared.data(from: url)
    guard let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) else { throw URLError(.badServerResponse) }
    return try JSONSerialization.jsonObject(with: data)
  }
}

struct ItemScreen: View {
  let category: String
  let slug: String
  let title: String
  @State private var item: [String: Any]?
  @State private var error: String?
  var body: some View {
    Group {
      if let item { List { JsonRow(value: item) } }
      else if let error { ContentUnavailableView("Unable to load", systemImage: "wifi.slash", description: Text(error)) }
      else { ProgressView() }
    }.navigationTitle(title).task {
      var components = URLComponents(string: "https://wfhub-api.cagatayldzz.com")!
      components.path = "/api/\(category)/\(slug).json"
      do {
        guard let url = components.url else { throw URLError(.badURL) }
        item = try await API.fetch(url) as? [String: Any]
        if item == nil { throw URLError(.cannotParseResponse) }
      } catch { self.error = error.localizedDescription }
    }
  }
}

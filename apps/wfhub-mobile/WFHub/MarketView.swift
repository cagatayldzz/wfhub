import SwiftUI

struct MarketView: View {
  @State private var orders: [[String: Any]] = []
  @State private var items: [String: [String: Any]] = [:]
  @State private var type = "sell"
  @State private var error: String?
  @State private var loading = true
  var body: some View {
    VStack {
      Picker("Order type", selection: $type) { Text("Sell").tag("sell"); Text("Buy").tag("buy") }.pickerStyle(.segmented).padding()
      if loading { Spacer(); ProgressView(); Spacer() }
      else if let error { ContentUnavailableView("Unable to load", systemImage: "wifi.slash", description: Text(error)) }
      else {
        List(Array(orders.filter { ($0["type"] as? String) == type }.prefix(35).enumerated()), id: \.offset) { _, order in
          let item = items[order["itemId"] as? String ?? ""]
          let language = (item?["i18n"] as? [String: Any])?["en"] as? [String: Any]
          let name = language?["name"] as? String ?? "Item"
          let slug = item?["slug"] as? String ?? ""
          let user = order["user"] as? [String: Any]
          if let url = URL(string: "https://warframe.market/items/\(slug)"), !slug.isEmpty {
            Link(destination: url) { orderLabel(name: name, order: order, user: user) }
          } else { orderLabel(name: name, order: order, user: user) }
        }.refreshable { await load() }
      }
    }.navigationTitle("Market").task { await load() }
  }
  private func orderLabel(name: String, order: [String: Any], user: [String: Any]?) -> some View {
    VStack(alignment: .leading) {
      Text(name).font(.headline)
      Text("\(user?["ingameName"] as? String ?? "") · \(order["platinum"] as? Int ?? 0) platinum · ×\(order["quantity"] as? Int ?? 0)").font(.caption).foregroundStyle(.secondary)
    }
  }
  private func load() async {
    do {
      async let orderResponse = API.fetch(URL(string: "https://api.warframe.market/v2/orders/recent")!)
      async let itemResponse = API.fetch(URL(string: "https://api.warframe.market/v2/items")!)
      let (rawOrders, rawItems) = try await (orderResponse, itemResponse)
      guard let orderData = (rawOrders as? [String: Any])?["data"] as? [[String: Any]], let itemData = (rawItems as? [String: Any])?["data"] as? [[String: Any]] else { throw URLError(.cannotParseResponse) }
      orders = orderData
      items = Dictionary(itemData.compactMap { item in (item["id"] as? String).map { ($0, item) } }, uniquingKeysWith: { first, _ in first })
      error = nil
    } catch { self.error = error.localizedDescription }
    loading = false
  }
}

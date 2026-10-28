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
      else if let error {
        ContentUnavailableView { Label("Unable to load", systemImage: "wifi.slash") } description: { Text(error) } actions: { Button("Retry") { Task { await load() } } }
      }
      else {
        ScrollView {
          LazyVStack(spacing: 10) {
            ForEach(Array(orders.filter { ($0["type"] as? String) == type }.prefix(35).enumerated()), id: \.offset) { _, order in
              let item = items[order["itemId"] as? String ?? ""]
              let language = (item?["i18n"] as? [String: Any])?["en"] as? [String: Any]
              let name = language?["name"] as? String ?? "Item"
              let slug = item?["slug"] as? String ?? ""
              let user = order["user"] as? [String: Any]
              if let url = URL(string: "https://warframe.market/items/\(slug)"), !slug.isEmpty {
                Link(destination: url) { orderLabel(name: name, order: order, user: user, icon: language?["icon"] as? String) }.buttonStyle(.plain)
              } else { orderLabel(name: name, order: order, user: user, icon: language?["icon"] as? String) }
            }
          }.padding(.horizontal, 12).padding(.bottom, 16)
        }.refreshable { await load() }
      }
    }.navigationTitle("Market").task {
      await load()
      while !Task.isCancelled {
        do { try await Task.sleep(nanoseconds: 60_000_000_000) } catch { return }
        await load()
      }
    }
  }
  private func orderLabel(name: String, order: [String: Any], user: [String: Any]?, icon: String?) -> some View {
    HStack(spacing: 0) {
      VStack(spacing: 0) {
        AsyncImage(url: icon.flatMap { URL(string: "https://warframe.market/static/assets/" + $0) }) { image in image.resizable().scaledToFit() } placeholder: { Color.clear }
          .frame(width: 62, height: 70).padding(8)
        Text(user?["ingameName"] as? String ?? "–").font(.caption.bold()).lineLimit(1)
          .foregroundStyle(.black).frame(maxWidth: .infinity).padding(.vertical, 5).background(WFStyle.gold)
      }.frame(width: 100).background(WFStyle.artwork)
      VStack(alignment: .leading, spacing: 7) {
        Text(name).font(.headline).foregroundStyle(.primary).lineLimit(2)
        Text("Total: \(order["quantity"] as? Int ?? 0)").font(.subheadline).foregroundStyle(WFStyle.muted)
        Label("\(order["platinum"] as? Int ?? 0)", systemImage: "diamond.fill").font(.subheadline.bold()).foregroundStyle(.green)
      }.padding(14).frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
    }.frame(minHeight: 112).background(Color(uiColor: .secondarySystemBackground))
      .clipShape(RoundedRectangle(cornerRadius: 10)).overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(Color.primary.opacity(0.07)))
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

import SwiftUI

struct MarketView: View {
  @State private var orders: [[String: Any]] = []
  @State private var items: [String: [String: Any]] = [:]
  @State private var type = "sell"
  @State private var error: String?
  @State private var loading = true
  var body: some View {
    ScrollView {
      LazyVStack(alignment: .leading, spacing: 10) {
        Text("Market").font(.system(size: 28, weight: .heavy)).padding(.top, 12)
        Picker("Order type", selection: $type) {
          Text("Sell").tag("sell")
          Text("Buy").tag("buy")
        }.pickerStyle(.segmented).padding(.vertical, 8)
        if loading {
          ProgressView().frame(maxWidth: .infinity).padding(.vertical, 40)
        } else if let error {
          ContentUnavailableView { Label("Unable to load", systemImage: "wifi.slash") } description: { Text(error) } actions: { Button("Retry") { Task { await load() } } }
        } else {
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
        }
      }.padding(.horizontal, 16).padding(.bottom, 16)
    }.refreshable { await load() }.navigationBarHidden(true).task {
      await load()
      while !Task.isCancelled {
        do { try await Task.sleep(nanoseconds: 60_000_000_000) } catch { return }
        await load()
      }
    }
  }
  private func orderLabel(name: String, order: [String: Any], user: [String: Any]?, icon: String?) -> some View {
    HStack(alignment: .center, spacing: 14) {
      AsyncImage(url: icon.flatMap { URL(string: "https://warframe.market/static/assets/" + $0) }) { image in
        image.resizable().scaledToFit().padding(8)
      } placeholder: {
        Image(systemName: "shippingbox").font(.title2).foregroundStyle(.white.opacity(0.5))
      }
      .frame(width: 76, height: 76).background(WFStyle.artwork).clipShape(RoundedRectangle(cornerRadius: 10))

      VStack(alignment: .leading, spacing: 7) {
        Text(name).font(.system(size: 16, weight: .bold)).foregroundStyle(.primary)
          .lineLimit(2).frame(maxWidth: .infinity, alignment: .leading)
        HStack(spacing: 5) {
          Image(systemName: "person.crop.circle").font(.caption)
          Text(user?["ingameName"] as? String ?? "–").lineLimit(1)
        }.font(.caption).foregroundStyle(.secondary)
        Text("×\(order["quantity"] as? Int ?? 0)").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
      }.frame(maxWidth: .infinity, alignment: .leading)

      VStack(alignment: .trailing, spacing: 5) {
        HStack(spacing: 4) {
          Image(systemName: "diamond.fill").font(.caption2)
          Text("\(order["platinum"] as? Int ?? 0)").font(.system(size: 17, weight: .bold)).monospacedDigit()
        }.foregroundStyle(WFStyle.accent)
        Text("PLATINUM").font(.system(size: 9, weight: .bold)).tracking(0.5).foregroundStyle(.secondary)
      }.fixedSize(horizontal: true, vertical: false)
    }
    .padding(12).frame(maxWidth: .infinity, alignment: .leading)
    .background(Color(uiColor: .secondarySystemBackground))
    .clipShape(RoundedRectangle(cornerRadius: 12))
    .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Color.primary.opacity(0.07)))
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

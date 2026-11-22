import SwiftUI

struct MarketView: View {
  @State private var book = MarketBook()
  @State private var rows = MarketRows(sell: [], buy: [])
  @State private var type = "sell"
  @State private var error: String?
  @State private var loading = true
  @State private var isRefreshing = false
  var body: some View {
    ScrollView {
      LazyVStack(alignment: .leading, spacing: 10) {
        Text(t("market")).font(.system(size: 28, weight: .heavy)).padding(.top, 12)
        Picker(t("orderType"), selection: $type) {
          Text(t("sell")).tag("sell")
          Text(t("buy")).tag("buy")
        }.pickerStyle(.segmented).padding(.vertical, 8)
        if loading {
          ProgressView().frame(maxWidth: .infinity).padding(.vertical, 40)
        } else if let error {
          ContentUnavailableView {
            Label(t("unableToLoad"), systemImage: "wifi.slash")
          } description: {
            Text(error)
          } actions: {
            Button(t("retry")) { Task { await load() } }
          }
        } else {
          ForEach(Array((type == "sell" ? rows.sell : rows.buy).enumerated()), id: \.offset) { _, row in
            if let link = row.link, let url = URL(string: link) {
              Link(destination: url) { orderLabel(row) }.buttonStyle(.plain)
            } else {
              orderLabel(row)
            }
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
  private func orderLabel(_ row: OrderRow) -> some View {
    HStack(alignment: .center, spacing: 14) {
      ThumbnailImage(url: row.icon.flatMap { URL(string: $0) }, maxPixelSize: 256)
        .padding(8)
        .frame(width: 76, height: 76).background(WFStyle.artwork).clipShape(RoundedRectangle(cornerRadius: 10))

      VStack(alignment: .leading, spacing: 7) {
        Text(row.name).font(.system(size: 16, weight: .bold)).foregroundStyle(.primary)
          .lineLimit(2).frame(maxWidth: .infinity, alignment: .leading)
        HStack(spacing: 5) {
          Image(systemName: "person.crop.circle").font(.caption)
          Text(row.user).lineLimit(1)
        }.font(.caption).foregroundStyle(.secondary)
        Text("×\(row.quantity)").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
      }.frame(maxWidth: .infinity, alignment: .leading)

      VStack(alignment: .trailing, spacing: 5) {
        HStack(spacing: 4) {
          Image(systemName: "diamond.fill").font(.caption2)
          Text("\(row.platinum)").font(.system(size: 17, weight: .bold)).monospacedDigit()
        }.foregroundStyle(WFStyle.accent)
        Text(t("platinum")).font(.system(size: 9, weight: .bold)).tracking(0.5).foregroundStyle(.secondary)
      }.fixedSize(horizontal: true, vertical: false)
    }
    .padding(12).frame(maxWidth: .infinity, alignment: .leading)
    .background(Color(uiColor: .secondarySystemBackground))
    .clipShape(RoundedRectangle(cornerRadius: 12))
    .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Color.primary.opacity(0.07)))
  }

  private func load() async {
    guard !isRefreshing else { return }
    isRefreshing = true
    defer { isRefreshing = false }
    do {
      let now = Int64(Date().timeIntervalSince1970)
      async let orders = API.fetchText(URL(string: marketOrdersUrl())!)
      async let catalogue: String? = book.needsCatalogue(nowUnix: now) ? API.fetchText(URL(string: marketItemsUrl())!) : nil
      let (ordersBody, catalogueBody) = try await (orders, catalogue)
      guard !Task.isCancelled else { return }
      if let catalogueBody { try book.setCatalogue(body: catalogueBody, nowUnix: now) }
      rows = try book.rows(ordersBody: ordersBody)
      error = nil
    } catch {
      guard !Task.isCancelled else { return }
      self.error = error.localizedDescription
    }
    loading = false
  }
}

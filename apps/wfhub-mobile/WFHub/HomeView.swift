import SwiftUI

private let gold = Color(red: 0.83, green: 0.73, blue: 0.44)
private let cdn = "https://cdn.cagatayldzz.com/constants/wfhub/"

private struct HomeData {
  var nightwave: [String: Any]?
  var trader: [String: Any]?
  var events: [[String: Any]] = []
  var alerts: [[String: Any]] = []
  var news: [[String: Any]] = []
  var banners: [[String: Any]] = []
  var failed = Set<String>()
}

struct HomeView: View {
  @EnvironmentObject private var ads: AdsManager
  @AppStorage("tennoName") private var name = "Tenno"
  @AppStorage("platform") private var platform = "pc"
  @Environment(\.locale) private var locale
  @State private var data = HomeData()
  @State private var loading = true

  private var turkish: Bool { locale.language.languageCode?.identifier == "tr" }
  private func text(_ english: String, _ turkishText: String) -> String {
    let language = locale.language.languageCode?.identifier ?? "en"
    guard let path = Bundle.main.path(forResource: language, ofType: "lproj"), let bundle = Bundle(path: path) else { return english }
    return bundle.localizedString(forKey: english, value: english, table: nil)
  }
  private func route(_ endpoint: String, fallback: String) -> Route {
    Catalogue.routes.activities.first { $0.endpoint == endpoint } ?? Route(title: fallback, endpoint: endpoint)
  }

  var body: some View {
    ScrollView {
      LazyVStack(alignment: .leading, spacing: 0) {
        Text("\(text("Welcome", "Hoş geldin")), \(name)")
          .font(.system(size: 28, weight: .heavy))
          .padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 12)

        ForEach(Array(data.banners.enumerated()), id: \.offset) { _, banner in
          if let title = banner["title"] as? String,
             let link = banner["link"] as? String,
             let url = URL(string: link.replacingOccurrences(of: "{lang}", with: locale.language.languageCode?.identifier ?? "en")),
             ["http", "https"].contains(url.scheme?.lowercased() ?? "") {
            Link(destination: url) {
              ImageBanner(image: banner["imageLink"] as? String, title: title, subtitle: nil, expiry: nil, imageAlignment: .leading)
            }.buttonStyle(.plain)
          }
        }

        if let nightwave = data.nightwave {
          NavigationLink { DataScreen(route: route("nightwave", fallback: "Nightwave"), kind: .activity) } label: {
            VStack(alignment: .leading, spacing: 6) {
              Text("NIGHTWAVE").font(.system(size: 26, weight: .black)).foregroundStyle(WFStyle.accent)
              Text("Season \(number(nightwave["season"]))")
              Countdown(expiry: nightwave["expiry"] as? String)
            }.frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 20).padding(.vertical, 24)
          }.buttonStyle(.plain)
          Divider()
        }

        if !data.events.isEmpty {
          sectionHeader(text("Events", "Etkinlikler"), subtitle: text("Latest Events", "Son etkinlikler"), endpoint: "events")
          VStack(spacing: 12) {
            ForEach(Array(data.events.enumerated()), id: \.offset) { _, event in EventCard(event: event) }
          }.padding(.horizontal, 16)
        }

        if ads.ready {
          HomeAdBanner()
            .padding(.horizontal, 16)
            .padding(.top, 24)
            .padding(.bottom, 8)
        }

        if !data.news.isEmpty {
          sectionHeader(text("News", "Haberler"), subtitle: text("Latest news", "Son haberler"), endpoint: "news")
          VStack(spacing: 15) {
            ForEach(Array(data.news.prefix(2).enumerated()), id: \.offset) { index, item in
              NewsCard(item: item, index: index + 1, language: locale.language.languageCode?.identifier ?? "en")
            }
          }.padding(.horizontal, 16).padding(.bottom, 16)
        }

        if let trader = data.trader {
          NavigationLink { DataScreen(route: route("voidTrader", fallback: "Baro Ki'Teer"), kind: .activity) } label: {
            ImageBanner(image: cdn + "sections/baro-ki-teer.png", title: "Baro Ki'Teer", subtitle: trader["location"] as? String, expiry: trader["expiry"] as? String, height: 140, imageAlignment: .leading)
          }.buttonStyle(.plain)
        }

        if !data.alerts.isEmpty {
          sectionHeader(text("Activities", "Aktiviteler"), subtitle: text("Latest Activities", "Son aktiviteler"), endpoint: "alerts")
          VStack(spacing: 12) {
            ForEach(Array(data.alerts.enumerated()), id: \.offset) { _, alert in AlertCard(alert: alert) }
          }.padding(.horizontal, 16)
        }

        sectionHeader("Codex", subtitle: text("Codex content", "Codex içeriği"), endpoint: nil)
        HStack(spacing: 10) {
          codexCard("Warframes", category: "warframes", image: "codex/warframes.png")
          codexCard("Secondary", category: "secondary", image: "codex/secondary.png")
        }.padding(.horizontal, 16).padding(.bottom, 32)

        if loading { ProgressView().frame(maxWidth: .infinity).padding() }
        if !data.failed.isEmpty {
          VStack(spacing: 8) {
            Text(text("Some content could not be loaded.", "Bazı içerikler yüklenemedi.")).font(.footnote).foregroundStyle(.secondary)
            Button(text("Retry", "Tekrar dene")) { Task { await load() } }
          }.frame(maxWidth: .infinity).padding(.bottom, 24)
        }
      }
    }
    .navigationBarHidden(true)
    .refreshable { await load() }
    .task(id: platform) {
      await load()
      while !Task.isCancelled {
        do { try await Task.sleep(nanoseconds: 60_000_000_000) } catch { return }
        await load()
      }
    }
  }

  @ViewBuilder private func sectionHeader(_ title: String, subtitle: String, endpoint: String?) -> some View {
    HStack(alignment: .bottom) {
      VStack(alignment: .leading, spacing: 4) {
        Text(title).font(.title2.bold())
        Text(subtitle).font(.subheadline).foregroundStyle(.secondary)
      }
      Spacer()
      if let endpoint {
        NavigationLink {
          DataScreen(route: Route(title: title, endpoint: endpoint), kind: .activity)
        } label: {
          Text(text("View all", "Tümünü gör"))
            .frame(minWidth: 70, minHeight: 44, alignment: .trailing)
            .contentShape(Rectangle())
        }
      } else {
        NavigationLink { CodexCategoriesView() } label: {
          Text(text("View all", "Tümünü gör"))
            .frame(minWidth: 70, minHeight: 44, alignment: .trailing)
            .contentShape(Rectangle())
        }
      }
    }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 16)
  }

  private func codexCard(_ title: String, category: String, image: String) -> some View {
    NavigationLink { DataScreen(route: Route(title: title, endpoint: category), kind: .category) } label: {
      ArtworkCard(title: title, image: URL(string: cdn + image))
    }.buttonStyle(.plain)
  }

  @MainActor private func load() async {
    let selectedPlatform = platform
    let endpoints = ["nightwave", "voidTrader", "events", "alerts", "news", "banners"]
    let results = await withTaskGroup(of: (String, Any?).self, returning: [(String, Any?)].self) { group in
      for endpoint in endpoints {
        group.addTask {
          let address = endpoint == "banners" ? cdn + "api/events.json" : "https://api.warframestat.us/\(selectedPlatform)/\(endpoint)/"
          guard let url = URL(string: address) else { return (endpoint, nil) }
          return (endpoint, try? await API.fetch(url))
        }
      }
      var results: [(String, Any?)] = []
      for await result in group { results.append(result) }
      return results
    }
    guard !Task.isCancelled, platform == selectedPlatform else { return }
    var updated = data
    updated.failed = []
    for (endpoint, value) in results {
      switch endpoint {
      case "nightwave": if let value = value as? [String: Any] { updated.nightwave = value } else { updated.failed.insert(endpoint) }
      case "voidTrader": if let value = value as? [String: Any] { updated.trader = value } else { updated.failed.insert(endpoint) }
      case "events": if let value = value as? [[String: Any]] { updated.events = value } else { updated.failed.insert(endpoint) }
      case "alerts": if let value = value as? [[String: Any]] { updated.alerts = value } else { updated.failed.insert(endpoint) }
      case "news": if let value = value as? [[String: Any]] { updated.news = value.sorted { ($0["date"] as? String ?? "") > ($1["date"] as? String ?? "") } } else { updated.failed.insert(endpoint) }
      case "banners": if let value = value as? [[String: Any]] { updated.banners = value } else { updated.failed.insert(endpoint) }
      default: break
      }
    }
    data = updated
    loading = false
  }
}

private func number(_ value: Any?) -> String {
  (value as? NSNumber)?.stringValue ?? "–"
}

private struct RemoteImage: View {
  let url: String?
  var alignment: Alignment = .center
  var body: some View {
    GeometryReader { geometry in
      AsyncImage(url: url.flatMap(URL.init(string:))) { image in image.resizable().scaledToFill() } placeholder: { Color.gray.opacity(0.2) }
        .frame(width: geometry.size.width, height: geometry.size.height, alignment: alignment)
        .clipped()
    }
  }
}

private struct ImageBanner: View {
  let image: String?
  let title: String
  let subtitle: String?
  let expiry: String?
  var height: CGFloat = 110
  var imageAlignment: Alignment = .center
  var body: some View {
    ZStack(alignment: .leading) {
      RemoteImage(url: image, alignment: imageAlignment).frame(maxWidth: .infinity).frame(height: height).clipped()
      Color.black.opacity(0.5)
      VStack(alignment: .leading, spacing: 5) {
        Text(title.uppercased()).font(.system(size: 22, weight: .black)).foregroundStyle(gold)
        if let subtitle, !subtitle.isEmpty { Text(subtitle).font(.body) }
        if expiry != nil { Countdown(expiry: expiry) }
      }.foregroundStyle(.white).padding(.horizontal, 20)
    }.frame(height: height).clipped().clipShape(RoundedRectangle(cornerRadius: 12)).padding(.horizontal, 16).padding(.vertical, 6)
  }
}

private struct Countdown: View {
  let expiry: String?
  var body: some View {
    TimelineView(.periodic(from: .now, by: 60)) { context in
      if let expiry, let date = parseDate(expiry) {
        let seconds = max(0, Int(date.timeIntervalSince(context.date)))
        Text("\(seconds / 86400)d \((seconds % 86400) / 3600)h \((seconds % 3600) / 60)m").fontWeight(.bold)
      }
    }
  }
}

private func parseDate(_ value: String) -> Date? {
  let formatter = ISO8601DateFormatter()
  formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
  if let date = formatter.date(from: value) { return date }
  formatter.formatOptions = [.withInternetDateTime]
  return formatter.date(from: value)
}

private struct EventCard: View {
  let event: [String: Any]
  private var progress: Double? {
    if let current = event["currentScore"] as? Double, let maximum = event["maximumScore"] as? Double, maximum > 0 {
      return min(100, max(0, 100 - current / maximum * 100))
    }
    if let health = event["health"] as? Double { return min(100, max(0, health)) }
    return nil
  }
  var body: some View {
    WFCard { VStack(alignment: .leading, spacing: 6) {
      HStack {
        Text(event["description"] as? String ?? "Event").font(.headline).lineLimit(2)
        Spacer()
        if let progress { Text(String(format: "%.2f%%", progress)).font(.caption.bold()) }
      }
      HStack {
        Text(event["node"] as? String ?? event["victimNode"] as? String ?? "–").foregroundStyle(.secondary)
        Spacer()
        Countdown(expiry: event["expiry"] as? String).font(.subheadline)
      }
    } }
  }
}

private struct AlertCard: View {
  let alert: [String: Any]
  var body: some View {
    let mission = alert["mission"] as? [String: Any] ?? [:]
    let reward = mission["reward"] as? [String: Any] ?? [:]
    let items = reward["items"] as? [String] ?? []
    let counted = (reward["countedItems"] as? [[String: Any]] ?? []).map { "\(number($0["count"])) \($0["type"] as? String ?? "")" }
    return WFCard { VStack(alignment: .leading, spacing: 6) {
      Text((alert["tag"] as? String) == "LotusGift" ? "Gifts of the Lotus" : (mission["node"] as? String ?? "Alert")).font(.headline)
      HStack {
        Text((items.isEmpty ? counted : items).joined(separator: ", ")).foregroundStyle(.secondary)
        Spacer()
        Countdown(expiry: alert["expiry"] as? String).font(.subheadline)
      }
    } }
  }
}

private struct NewsCard: View {
  let item: [String: Any]
  let index: Int
  let language: String
  var body: some View {
    let translations = item["translations"] as? [String: String] ?? [:]
    let title = translations[language] ?? translations["en"] ?? item["message"] as? String ?? "News"
    let content = VStack(alignment: .leading, spacing: 8) {
      ZStack(alignment: .topLeading) {
        RemoteImage(url: item["imageLink"] as? String).frame(height: 150).clipped().cornerRadius(5)
        Text("\(index)").font(.caption.bold()).foregroundStyle(.black).padding(5).background(.white, in: Circle()).padding(10)
      }
      Text(title.uppercased()).font(.system(size: 18, weight: .bold)).frame(maxWidth: .infinity, alignment: .leading)
      if let date = item["date"] as? String, let parsed = parseDate(date) {
        Text(parsed, format: .dateTime.day().month().year().hour().minute()).foregroundStyle(.secondary)
      }
    }
    if let link = item["link"] as? String, let url = URL(string: link), ["http", "https"].contains(url.scheme?.lowercased() ?? "") {
      Link(destination: url) { content }.buttonStyle(.plain)
    } else { content }
  }
}

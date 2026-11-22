import SwiftUI

private let gold = Color(red: 0.83, green: 0.73, blue: 0.44)

struct HomeView: View {
  @EnvironmentObject private var ads: AdsManager
  @AppStorage("tennoName") private var name = "Tenno"
  @AppStorage("platform") private var platform = "pc"
  @Environment(\.locale) private var locale
  @State private var data = HomeState(nightwave: nil, trader: nil, events: [], alerts: [], news: [], banners: [], failed: [])
  @State private var loading = true

  private func route(_ endpoint: String, fallback: String) -> Route {
    activityRoute(endpoint: endpoint, fallback: fallback)
  }

  var body: some View {
    ScrollView {
      LazyVStack(alignment: .leading, spacing: 0) {
        Text("\(t("welcome")), \(name)")
          .font(.system(size: 28, weight: .heavy))
          .padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 12)

        ForEach(Array(data.banners.enumerated()), id: \.offset) { _, banner in
          if let url = URL(string: banner.link) {
            Link(destination: url) {
              ImageBanner(image: banner.image, title: banner.title, subtitle: nil, expiry: nil, imageAlignment: .leading)
            }.buttonStyle(.plain)
          }
        }

        if let nightwave = data.nightwave {
          NavigationLink {
            DataScreen(route: route("nightwave", fallback: "Nightwave"), kind: .activity)
          } label: {
            VStack(alignment: .leading, spacing: 6) {
              Text(t("nightwave")).font(.system(size: 26, weight: .black)).foregroundStyle(WFStyle.accent)
              Text(tf("seasonNumber", Int(nightwave.season ?? 0)))
              Countdown(expiry: nightwave.expiry)
            }.frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 20).padding(.vertical, 24)
          }.buttonStyle(.plain)
          Divider()
        }

        if !data.events.isEmpty {
          sectionHeader(t("events"), subtitle: t("latestEvents"), endpoint: "events")
          VStack(spacing: 12) {
            ForEach(Array(data.events.enumerated()), id: \.offset) { _, event in EventRow(event: event) }
          }.padding(.horizontal, 16)
        }

        if ads.ready {
          HomeAdBanner()
            .padding(.horizontal, 16)
            .padding(.top, 24)
            .padding(.bottom, 8)
        }

        if !data.news.isEmpty {
          sectionHeader(t("news"), subtitle: t("latestNews"), endpoint: "news")
          VStack(spacing: 15) {
            ForEach(Array(data.news.enumerated()), id: \.offset) { index, item in
              NewsRow(item: item, index: index + 1)
            }
          }.padding(.horizontal, 16).padding(.bottom, 16)
        }

        if let trader = data.trader {
          NavigationLink {
            DataScreen(route: route("voidTrader", fallback: "Baro Ki'Teer"), kind: .activity)
          } label: {
            ImageBanner(
              image: publicUrl(path: "sections/baro-ki-teer.png"), title: "Baro Ki'Teer", subtitle: trader.location,
              expiry: trader.expiry, height: 140, imageAlignment: .leading)
          }.buttonStyle(.plain)
        }

        if !data.alerts.isEmpty {
          sectionHeader(t("activities"), subtitle: t("latestActivities"), endpoint: "alerts")
          VStack(spacing: 12) {
            ForEach(Array(data.alerts.enumerated()), id: \.offset) { _, alert in AlertRow(alert: alert) }
          }.padding(.horizontal, 16)
        }

        sectionHeader(t("codex"), subtitle: t("codexContent"), endpoint: nil)
        HStack(spacing: 10) {
          codexCard(t("warframes"), category: "warframes", image: "codex/warframes.png")
          codexCard(t("secondary"), category: "secondary", image: "codex/secondary.png")
        }.padding(.horizontal, 16).padding(.bottom, 32)

        if loading { ProgressView().frame(maxWidth: .infinity).padding() }
        if !data.failed.isEmpty {
          VStack(spacing: 8) {
            Text(t("someContentFailedToLoad")).font(.footnote).foregroundStyle(.secondary)
            Button(t("retry")) { Task { await load() } }
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
          Text(t("viewAll"))
            .frame(minWidth: 70, minHeight: 44, alignment: .trailing)
            .contentShape(Rectangle())
        }
      } else {
        NavigationLink {
          CodexCategoriesView()
        } label: {
          Text(t("viewAll"))
            .frame(minWidth: 70, minHeight: 44, alignment: .trailing)
            .contentShape(Rectangle())
        }
      }
    }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 16)
  }

  private func codexCard(_ title: String, category: String, image: String) -> some View {
    NavigationLink {
      DataScreen(route: Route(title: title, endpoint: category), kind: .category)
    } label: {
      ArtworkCard(title: title, image: URL(string: publicUrl(path: image)))
    }.buttonStyle(.plain)
  }

  @MainActor private func load() async {
    let selectedPlatform = platform
    let language = resolveLanguage(preferred: [locale.language.languageCode?.identifier ?? "en"])
    let responses = await withTaskGroup(of: HomeResponse.self, returning: [HomeResponse].self) { group in
      for request in homeRequests(platform: selectedPlatform) {
        group.addTask {
          guard let url = URL(string: request.url) else { return HomeResponse(key: request.key, body: nil) }
          return HomeResponse(key: request.key, body: try? await API.fetchText(url))
        }
      }
      var responses: [HomeResponse] = []
      for await response in group { responses.append(response) }
      return responses
    }
    guard !Task.isCancelled, platform == selectedPlatform else { return }
    data = parseHome(responses: responses, language: language)
    loading = false
  }
}

private struct RemoteImage: View {
  let url: String?
  var alignment: Alignment = .center
  var body: some View {
    GeometryReader { geometry in
      ThumbnailImage(url: url.flatMap(URL.init(string:)), contentMode: .fill)
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

struct Countdown: View {
  let expiry: String?
  var body: some View {
    TimelineView(.periodic(from: .now, by: 60)) { context in
      if let expiry, let text = countdown(expiry: expiry, nowUnix: Int64(context.date.timeIntervalSince1970)) {
        Text(text).fontWeight(.bold)
      }
    }
  }
}

private struct EventRow: View {
  let event: EventCard
  var body: some View {
    WFCard {
      VStack(alignment: .leading, spacing: 6) {
        HStack {
          Text(event.title).font(.headline).lineLimit(2)
          Spacer()
          if let progress = event.progress { Text(String(format: "%.2f%%", progress)).font(.caption.bold()) }
        }
        HStack {
          Text(event.node).foregroundStyle(.secondary)
          Spacer()
          Countdown(expiry: event.expiry).font(.subheadline)
        }
      }
    }
  }
}

private struct AlertRow: View {
  let alert: AlertCard
  var body: some View {
    WFCard {
      VStack(alignment: .leading, spacing: 6) {
        Text(alert.title).font(.headline)
        HStack {
          Text(alert.rewards).foregroundStyle(.secondary)
          Spacer()
          Countdown(expiry: alert.expiry).font(.subheadline)
        }
      }
    }
  }
}

private struct NewsRow: View {
  let item: NewsCard
  let index: Int
  var body: some View {
    let content = VStack(alignment: .leading, spacing: 8) {
      ZStack(alignment: .topLeading) {
        RemoteImage(url: item.image).frame(height: 150).clipped().cornerRadius(5)
        Text("\(index)").font(.caption.bold()).foregroundStyle(.black).padding(5).background(.white, in: Circle()).padding(10)
      }
      Text(item.title.uppercased()).font(.system(size: 18, weight: .bold)).frame(maxWidth: .infinity, alignment: .leading)
      if let date = item.date.flatMap(expiryUnix).map({ Date(timeIntervalSince1970: TimeInterval($0)) }) {
        Text(date, format: .dateTime.day().month().year().hour().minute()).foregroundStyle(.secondary)
      }
    }
    if let link = item.link, let url = URL(string: link) {
      Link(destination: url) { content }.buttonStyle(.plain)
    } else {
      content
    }
  }
}

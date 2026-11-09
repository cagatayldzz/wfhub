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
            Text(t("activities")).font(.system(size: 28, weight: .heavy))
            Text(t("worldstateDetails")).font(.subheadline).foregroundStyle(.secondary)
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
      ThumbnailImage(url: URL(string: "https://wfhub.cagatayldzz.com/public/activities/\(imageName).png"), maxPixelSize: 160)
      .frame(width: 40, height: 40)
      VStack(alignment: .leading, spacing: 3) {
        Text(td(route.title)).font(.system(size: 16, weight: .semibold)).foregroundStyle(.primary)
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

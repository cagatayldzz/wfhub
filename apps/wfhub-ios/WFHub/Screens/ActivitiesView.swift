import SwiftUI

struct ActivitiesView: View {
  private let routes = activityRoutes()

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
            NavigationLink {
              CyclesView()
            } label: {
              ActivityLink(route: route)
            }.buttonStyle(.plain)
          } else {
            NavigationLink {
              DataScreen(route: route, kind: .activity)
            } label: {
              ActivityLink(route: route)
            }.buttonStyle(.plain)
          }
        }
      }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 24)
    }
    .navigationBarHidden(true)
  }
}

private struct ActivityLink: View {
  let route: Route
  var body: some View {
    HStack(spacing: 12) {
      ThumbnailImage(url: URL(string: activityImageUrl(endpoint: route.endpoint)), maxPixelSize: 160)
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

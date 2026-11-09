import SwiftUI

struct CodexCategoriesView: View {
  var body: some View {
    ScrollView {
      VStack(spacing: 10) {
        NavigationLink { MasteryChecklistView() } label: {
          CodexMenuCard(title: t("masteryRankChecklist"), subtitle: t("warframes"), icon: "checklist")
        }.buttonStyle(.plain)
        NavigationLink { CodexCatalogueView() } label: {
          CodexMenuCard(title: t("codexCatalogue"), subtitle: t("browseCategories"), icon: "book.closed")
        }.buttonStyle(.plain)
      }.padding(10)
    }.navigationTitle(t("codex"))
  }
}

private struct CodexMenuCard: View {
  let title: String
  let subtitle: String
  let icon: String

  var body: some View {
    HStack(spacing: 14) {
      Image(systemName: icon).font(.title2).foregroundStyle(WFStyle.accent).frame(width: 30)
      VStack(alignment: .leading, spacing: 4) {
        Text(title).font(.headline)
        Text(subtitle).font(.subheadline).foregroundStyle(.secondary)
      }
      Spacer()
      Image(systemName: "chevron.right").font(.caption.bold()).foregroundStyle(.secondary)
    }.padding(16).background(Color(uiColor: .secondarySystemBackground), in: RoundedRectangle(cornerRadius: 12))
  }
}

private struct CodexCatalogueView: View {
  private let columns = Array(repeating: GridItem(.flexible(), spacing: 10), count: 2)
  var body: some View {
    ScrollView {
      LazyVGrid(columns: columns, spacing: 10) {
        ForEach(Catalogue.routes.categories.sorted { $0.title.localizedCaseInsensitiveCompare($1.title) == .orderedAscending }) { route in
          NavigationLink {
            DataScreen(route: route, kind: .category)
          } label: {
            ArtworkCard(title: route.title, image: URL(string: "https://wfhub.cagatayldzz.com/public/codex/\(route.endpoint == "misc" ? "gear" : route.endpoint).png"))
          }.buttonStyle(.plain)
        }
      }.padding(10)
    }.navigationTitle(t("codexCatalogue"))
  }
}

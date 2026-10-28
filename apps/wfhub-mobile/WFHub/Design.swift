import SwiftUI

// Shared visual language used by the native screens.
enum WFStyle {
  static let gold = Color(red: 0.88, green: 0.78, blue: 0.45)
  static let accent = Color(uiColor: UIColor { traits in
    traits.userInterfaceStyle == .dark ? UIColor(red: 0.88, green: 0.78, blue: 0.45, alpha: 1) : UIColor(red: 0.45, green: 0.33, blue: 0.04, alpha: 1)
  })
  static let muted = Color.secondary
  static let artwork = LinearGradient(colors: [Color(red: 0.08, green: 0.08, blue: 0.09), Color(red: 0.15, green: 0.18, blue: 0.21), .black], startPoint: .top, endPoint: .bottom)
}

struct WFCard<Content: View>: View {
  @ViewBuilder let content: Content
  var body: some View {
    content
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(16)
      .background(Color(uiColor: .secondarySystemBackground))
      .clipShape(RoundedRectangle(cornerRadius: 12))
      .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Color.primary.opacity(0.07)))
  }
}

struct ArtworkCard: View {
  let title: String
  let image: URL?
  var body: some View {
    VStack(spacing: 6) {
      AsyncImage(url: image) { image in image.resizable().scaledToFit() } placeholder: { Color.clear }
        .frame(height: 100).frame(maxWidth: .infinity)
      Text(title).font(.system(size: 15, weight: .bold)).foregroundStyle(.white)
        .lineLimit(2).multilineTextAlignment(.center).frame(maxWidth: .infinity, minHeight: 40)
    }
    .padding(10).frame(maxWidth: .infinity).background(WFStyle.artwork)
    .clipShape(RoundedRectangle(cornerRadius: 10))
    .contentShape(Rectangle())
  }
}

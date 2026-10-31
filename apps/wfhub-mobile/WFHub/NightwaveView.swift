import SwiftUI

// Nightwave screen: hero card with the season countdown, then one row per active challenge.
struct NightwaveView: View {
  let value: [String: Any]
  private let cdn = "https://cdn.cagatayldzz.com/constants/wfhub/"
  private let gold = Color(red: 0.83, green: 0.73, blue: 0.44)

  var body: some View {
    VStack(spacing: 0) {
      hero
      LazyVStack(spacing: 10) {
        ForEach(Array((value["activeChallenges"] as? [[String: Any]] ?? []).enumerated()), id: \.offset) { _, challenge in
          NightwaveChallengeRow(challenge: challenge)
        }
      }.padding(10)
    }
  }

  private var hero: some View {
    VStack(alignment: .leading, spacing: 10) {
      Text("NIGHTWAVE").font(.system(size: 26, weight: .black)).foregroundStyle(gold)
      if let season = value["season"] as? Int { Text("Season \(season)").font(.system(size: 16, weight: .medium)) }
      Countdown(expiry: value["expiry"] as? String).font(.system(size: 16))
    }
    .foregroundStyle(.white)
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(.horizontal, 20).padding(.vertical, 24)
    .background {
      ZStack {
        ThumbnailImage(url: URL(string: cdn + "sections/nightwave.jpg"), contentMode: .fill, maxPixelSize: 1200)
        Color.black.opacity(0.5)
      }
    }
    .clipShape(Rectangle())
  }
}

private struct NightwaveChallengeRow: View {
  let challenge: [String: Any]
  private let gold = Color(red: 0.83, green: 0.73, blue: 0.44)

  private var title: String { challenge["title"] as? String ?? "Challenge" }
  private var kind: String {
    if challenge["isDaily"] as? Bool == true { return "Daily" }
    return challenge["isElite"] as? Bool == true ? "Elite Weekly" : "Weekly"
  }
  // Same file naming as the CDN uses: no apostrophes, hyphenated, trailing roman numeral dropped.
  private var slug: String {
    var text = title.replacingOccurrences(of: "'", with: "").replacingOccurrences(of: "\\s+", with: "-", options: .regularExpression)
    text = text.replacingOccurrences(of: "(?:-\\s?|^)[IVXLCDMivxlcdm]+$", with: "", options: .regularExpression)
    return text.lowercased()
  }

  var body: some View {
    HStack(spacing: 0) {
      ZStack(alignment: .bottom) {
        LinearGradient(colors: [Color(red: 0.08, green: 0.08, blue: 0.08), Color(red: 0.15, green: 0.18, blue: 0.2), .black], startPoint: .bottom, endPoint: .top)
        ThumbnailImage(url: URL(string: "https://cdn.cagatayldzz.com/constants/wfhub/nightwave/\(slug).png"), maxPixelSize: 192, showsPlaceholder: false)
          .frame(width: 75, height: 75)
          .frame(maxHeight: .infinity)
        if let points = challenge["reputation"] as? Int {
          Text("\(points)").font(.system(size: 14, weight: .bold)).foregroundStyle(gold).padding(.vertical, 4)
        }
      }.frame(width: 100)

      VStack(alignment: .leading, spacing: 2) {
        Text(title).font(.system(size: 18, weight: .bold))
        Text(kind).font(.system(size: 14, weight: .bold)).foregroundStyle(gold)
        Text(challenge["desc"] as? String ?? "").font(.system(size: 15)).foregroundStyle(.secondary).padding(.top, 8)
      }
      .frame(maxWidth: .infinity, alignment: .leading)
      .padding(12)
      .overlay(Rectangle().stroke(Color.primary.opacity(0.15), lineWidth: 1))
    }
    .fixedSize(horizontal: false, vertical: true)
  }
}

import SwiftUI
import ImageIO

// Decoded thumbnails live in memory; downloaded files live on disk, so scrolling never re-downloads or re-decodes.
enum ImageStore {
  static let memory: NSCache<NSString, UIImage> = {
    let cache = NSCache<NSString, UIImage>()
    cache.totalCostLimit = 64 * 1024 * 1024
    return cache
  }()

  private static let session: URLSession = {
    let configuration = URLSessionConfiguration.default
    configuration.urlCache = URLCache(memoryCapacity: 16 * 1024 * 1024, diskCapacity: 300 * 1024 * 1024, directory: nil)
    configuration.requestCachePolicy = .returnCacheDataElseLoad
    return URLSession(configuration: configuration)
  }()

  static func key(_ url: URL, _ size: Int) -> NSString { "\(url.absoluteString)#\(size)" as NSString }

  static func cached(_ url: URL?, size: Int) -> UIImage? {
    url.flatMap { memory.object(forKey: key($0, size)) }
  }

  static func load(_ url: URL, size: Int) async -> UIImage? {
    if let hit = memory.object(forKey: key(url, size)) { return hit }
    guard let (data, response) = try? await session.data(from: url),
          (response as? HTTPURLResponse).map({ (200..<300).contains($0.statusCode) }) ?? false else { return nil }
    let decoded = await Task.detached(priority: .utility) { () -> UIImage? in
      guard let source = CGImageSourceCreateWithData(data as CFData, nil),
            let thumbnail = CGImageSourceCreateThumbnailAtIndex(source, 0, [
              kCGImageSourceCreateThumbnailFromImageAlways: true,
              kCGImageSourceCreateThumbnailWithTransform: true,
              kCGImageSourceThumbnailMaxPixelSize: size,
              kCGImageSourceShouldCacheImmediately: true
            ] as CFDictionary) else { return nil }
      return UIImage(cgImage: thumbnail)
    }.value
    if let decoded {
      memory.setObject(decoded, forKey: key(url, size), cost: decoded.cgImage.map { $0.bytesPerRow * $0.height } ?? 0)
    }
    return decoded
  }
}

// Decode remote artwork near its displayed size instead of expanding full-resolution images in memory.
struct ThumbnailImage: View {
  let url: URL?
  var contentMode: ContentMode = .fit
  var maxPixelSize: Int = 1200
  var showsPlaceholder = true
  @State private var image: UIImage?

  var body: some View {
    Group {
      if let image = image ?? ImageStore.cached(url, size: maxPixelSize) {
        Image(uiImage: image).resizable().aspectRatio(contentMode: contentMode)
      } else {
        // Keep a real view here: .task never runs on an empty Group.
        Color.gray.opacity(showsPlaceholder ? 0.2 : 0)
      }
    }
    .task(id: url) {
      guard let url else { image = nil; return }
      if let hit = ImageStore.cached(url, size: maxPixelSize) { image = hit; return }
      image = nil
      let loaded = await ImageStore.load(url, size: maxPixelSize)
      guard !Task.isCancelled else { return }
      image = loaded
    }
  }
}

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
      ThumbnailImage(url: image, maxPixelSize: 600)
        .frame(height: 100).frame(maxWidth: .infinity)
      Text(title).font(.system(size: 15, weight: .bold)).foregroundStyle(.white)
        .lineLimit(2).multilineTextAlignment(.center).frame(maxWidth: .infinity, minHeight: 40)
    }
    .padding(10).frame(maxWidth: .infinity).background(WFStyle.artwork)
    .clipShape(RoundedRectangle(cornerRadius: 10))
    .contentShape(Rectangle())
  }
}

import GoogleMobileAds
import SwiftUI
import UserMessagingPlatform

@MainActor
final class AdsManager: ObservableObject {
  @Published private(set) var ready = false
  @Published private(set) var privacyOptionsRequired = false
  private var started = false
  private var loadedWidth: CGFloat = 0
  private(set) lazy var banner: BannerView = {
    let view = BannerView()
    #if DEBUG
    view.adUnitID = "ca-app-pub-3940256099942544/2435281174"
    #else
    view.adUnitID = "ca-app-pub-6626336739486650/9106298353"
    #endif
    view.delegate = bannerDelegate
    return view
  }()
  private let bannerDelegate = BannerDelegate()

  static func bannerWidth() -> CGFloat {
    max(320, min(UIScreen.main.bounds.width - 32, 728))
  }

  // Loads once per width; the same BannerView is reused so scrolling never creates new ad requests.
  func loadBanner(width: CGFloat) {
    guard ready, width != loadedWidth else { return }
    loadedWidth = width
    banner.rootViewController = UIApplication.shared.connectedScenes
      .compactMap { ($0 as? UIWindowScene)?.keyWindow?.rootViewController }.first
    banner.adSize = currentOrientationAnchoredAdaptiveBanner(width: width)
    banner.load(Request())
  }

  func prepare() async {
    guard !started else { return }
    started = true
    do {
      try await ConsentInformation.shared.requestConsentInfoUpdate(with: RequestParameters())
      try await ConsentForm.loadAndPresentIfRequired(from: nil)
    } catch {
      print("Ad privacy update failed: \(error.localizedDescription)")
    }
    privacyOptionsRequired = ConsentInformation.shared.privacyOptionsRequirementStatus == .required
    if ConsentInformation.shared.canRequestAds {
      await MobileAds.shared.start()
      ready = true
    }
  }

  func showPrivacyOptions() async {
    do {
      try await ConsentForm.presentPrivacyOptionsForm(from: nil)
    } catch {
      print("Privacy options failed: \(error.localizedDescription)")
    }
    ready = ConsentInformation.shared.canRequestAds
  }
}

final class BannerDelegate: NSObject, BannerViewDelegate {
  func bannerViewDidReceiveAd(_ bannerView: BannerView) { print("Ad loaded") }
  func bannerView(_ bannerView: BannerView, didFailToReceiveAdWithError error: Error) {
    print("Ad failed: \(error.localizedDescription)")
  }
}

struct HomeAdBanner: View {
  @EnvironmentObject private var ads: AdsManager
  var body: some View {
    let width = AdsManager.bannerWidth()
    let height = currentOrientationAnchoredAdaptiveBanner(width: width).size.height
    BannerContainer(banner: ads.banner)
      .frame(width: width, height: height)
      .frame(maxWidth: .infinity)
      .onAppear { ads.loadBanner(width: width) }
  }
}

private struct BannerContainer: UIViewRepresentable {
  let banner: BannerView

  func makeUIView(context: Context) -> UIView {
    let host = UIView()
    banner.translatesAutoresizingMaskIntoConstraints = false
    host.addSubview(banner)
    NSLayoutConstraint.activate([
      banner.topAnchor.constraint(equalTo: host.topAnchor),
      banner.centerXAnchor.constraint(equalTo: host.centerXAnchor)
    ])
    return host
  }

  func updateUIView(_ host: UIView, context: Context) {}

  static func dismantleUIView(_ host: UIView, coordinator: ()) {
    host.subviews.forEach { $0.removeFromSuperview() }
  }
}

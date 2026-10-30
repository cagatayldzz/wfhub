import GoogleMobileAds
import SwiftUI
import UserMessagingPlatform

@MainActor
final class AdsManager: ObservableObject {
  @Published private(set) var ready = false
  @Published private(set) var privacyOptionsRequired = false
  private var started = false

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

struct HomeAdBanner: View {
  var body: some View {
    BannerContainer(size: AdSizeBanner)
      .frame(width: AdSizeBanner.size.width, height: AdSizeBanner.size.height)
      .frame(maxWidth: .infinity, minHeight: 66)
  }
}

private struct BannerContainer: UIViewRepresentable {
  let size: AdSize

  func makeUIView(context: Context) -> BannerView {
    let banner = BannerView(adSize: size)
    #if DEBUG
    banner.adUnitID = "ca-app-pub-3940256099942544/2435281174"
    #else
    banner.adUnitID = "ca-app-pub-6626336739486650/9106298353"
    #endif
    banner.load(Request())
    return banner
  }

  func updateUIView(_ banner: BannerView, context: Context) {
    if banner.adSize.size != size.size {
      banner.adSize = size
      banner.load(Request())
    }
  }
}

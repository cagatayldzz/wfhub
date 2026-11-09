import Foundation

let supportedLanguages = ["de", "en", "es", "fr", "it", "ja", "ko", "pl", "pt", "ru", "th", "tr", "uk", "zh"]

private enum TranslationBundles {
  static let english = bundle("en")
  private static var cache: [String: Bundle?] = [:]

  static func bundle(_ code: String) -> Bundle? {
    if let cached = cache[code] { return cached }
    let found = Bundle.main.path(forResource: code, ofType: "lproj").flatMap(Bundle.init(path:))
    cache[code] = found
    return found
  }
}

private func currentLanguageCode() -> String {
  let preference = UserDefaults.standard.string(forKey: "appLanguage") ?? "system"
  if preference != "system" { return preference }
  for identifier in Locale.preferredLanguages {
    if let code = supportedLanguages.first(where: { identifier.hasPrefix($0) }) { return code }
  }
  return "en"
}

func t(_ key: String) -> String {
  let fallback = TranslationBundles.english?.localizedString(forKey: key, value: key, table: nil) ?? key
  return TranslationBundles.bundle(currentLanguageCode())?.localizedString(forKey: key, value: fallback, table: nil) ?? fallback
}

func tf(_ key: String, _ arguments: CVarArg...) -> String {
  String(format: t(key), locale: Locale.current, arguments: arguments)
}

/// Translates text that comes from the API (English display text) by converting it into a camelCase key.
/// Falls back to the original text when no translation exists.
func td(_ text: String) -> String {
  let words = text.replacingOccurrences(of: "'", with: "").replacingOccurrences(of: "’", with: "")
    .split(whereSeparator: { !($0.isASCII && ($0.isLetter || $0.isNumber)) }).map(String.init)
  guard let first = words.first else { return text }
  let key = first.lowercased() + words.dropFirst().map { $0.prefix(1).uppercased() + $0.dropFirst().lowercased() }.joined()
  let english = TranslationBundles.english?.localizedString(forKey: key, value: text, table: nil) ?? text
  return TranslationBundles.bundle(currentLanguageCode())?.localizedString(forKey: key, value: english, table: nil) ?? english
}

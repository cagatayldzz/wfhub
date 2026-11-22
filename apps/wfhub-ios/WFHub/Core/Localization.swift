import Foundation

// The texts and the language rules live in the Rust core (`crates/wfhub-core`), shared with Android. The `.lproj` files stay in the
// bundle for SwiftUI's own lookups (`Text("key")`); the core embeds the same files.

private func currentLanguageCode() -> String {
  let preference = UserDefaults.standard.string(forKey: "appLanguage") ?? "system"
  return preference == "system" ? resolveLanguage(preferred: Locale.preferredLanguages) : preference
}

func t(_ key: String) -> String {
  translate(language: currentLanguageCode(), key: key)
}

func tf(_ key: String, _ arguments: CVarArg...) -> String {
  String(format: t(key), locale: Locale.current, arguments: arguments)
}

/// Translates text that comes from the API (English display text). Falls back to the original text when no translation exists.
func td(_ text: String) -> String {
  translateData(language: currentLanguageCode(), text: text)
}

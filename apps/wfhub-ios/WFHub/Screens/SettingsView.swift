import SwiftUI

struct SettingsView: View {
  @EnvironmentObject private var ads: AdsManager
  @AppStorage("tennoName") private var storedName = "Tenno"
  @AppStorage("platform") private var storedPlatform = "pc"
  @AppStorage("appLanguage") private var language = "system"
  @AppStorage("appTheme") private var theme = "system"
  @State private var name = ""
  @State private var platform = "pc"
  @State private var saved = false
  @FocusState private var editingName: Bool

  private let languages: [(String, String)] = [
    ("de", "Deutsch"), ("en", "English"), ("es", "Español"), ("fr", "Français"),
    ("it", "Italiano"), ("ja", "日本語"), ("ko", "한국어"), ("pl", "Polski"),
    ("pt", "Português"), ("ru", "Русский"), ("th", "ไทย"), ("tr", "Türkçe"),
    ("uk", "Українська"), ("zh", "简体中文")
  ]
  private let platforms: [(String, String)] = [("pc", "PC"), ("xb1", "Xbox"), ("swi", "Nintendo Switch"), ("ps4", "PlayStation")]

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 24) {
        Text(t("settings")).font(.system(size: 28, weight: .heavy))

        VStack(alignment: .leading, spacing: 10) {
          Text(t("profile")).font(.caption.bold()).tracking(1).foregroundStyle(.secondary).padding(.horizontal, 4)
          VStack(spacing: 0) {
            HStack(spacing: 12) {
              SettingsIcon(name: "person.crop.circle")
              VStack(alignment: .leading, spacing: 4) {
                Text(t("tennoName")).font(.subheadline).foregroundStyle(.secondary)
                TextField(t("tennoName"), text: $name).textInputAutocapitalization(.never).autocorrectionDisabled()
                  .font(.body.weight(.semibold)).focused($editingName)
              }
            }.padding(16)
            Divider().padding(.leading, 60)
            Menu {
              ForEach(platforms, id: \.0) { code, title in
                Button {
                  platform = code
                } label: {
                  if platform == code { Label(title, systemImage: "checkmark") } else { Text(title) }
                }
              }
            } label: {
              SettingsRow(icon: "gamecontroller", title: "platform", value: platforms.first { $0.0 == platform }?.1 ?? "PC")
            }
          }.tint(.primary).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
        }

        VStack(alignment: .leading, spacing: 10) {
          Text(t("preferences")).font(.caption.bold()).tracking(1).foregroundStyle(.secondary).padding(.horizontal, 4)
          VStack(spacing: 0) {
            Menu {
              Button {
                language = "system"
              } label: {
                if language == "system" { Label(t("system"), systemImage: "checkmark") } else { Text(t("system")) }
              }
              ForEach(languages, id: \.0) { code, title in
                Button {
                  language = code
                } label: {
                  if language == code { Label(title, systemImage: "checkmark") } else { Text(title) }
                }
              }
            } label: {
              SettingsRow(
                icon: "globe", title: "language", value: language == "system" ? "System" : languages.first { $0.0 == language }?.1 ?? "English")
            }
            Divider().padding(.leading, 60)
            Menu {
              Button {
                theme = "system"
              } label: {
                if theme == "system" { Label(t("system"), systemImage: "checkmark") } else { Text(t("system")) }
              }
              Button {
                theme = "light"
              } label: {
                if theme == "light" { Label(t("light"), systemImage: "checkmark") } else { Text(t("light")) }
              }
              Button {
                theme = "dark"
              } label: {
                if theme == "dark" { Label(t("dark"), systemImage: "checkmark") } else { Text(t("dark")) }
              }
            } label: {
              SettingsRow(
                icon: "circle.lefthalf.filled", title: "appearance", value: theme == "light" ? "Light" : theme == "dark" ? "Dark" : "System")
            }
          }.tint(.primary).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
        }

        VStack(spacing: 0) {
          NavigationLink {
            FeedbackView()
          } label: {
            SettingsLinkRow(icon: "bubble.left.and.bubble.right", title: "feedback")
          }
          Divider().padding(.leading, 60)
          Link(destination: URL(string: "https://wfhub.cagatayldzz.com/legal/privacy-policy")!) {
            SettingsLinkRow(icon: "hand.raised", title: "privacyPolicy")
          }
          if ads.privacyOptionsRequired {
            Divider().padding(.leading, 60)
            Button {
              Task { await ads.showPrivacyOptions() }
            } label: {
              SettingsLinkRow(icon: "hand.raised", title: "privacyOptions")
            }
          }
        }.tint(.primary).background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))

        Button {
          storedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
          storedPlatform = platform
          editingName = false
          saved = true
        } label: {
          Text(t("save")).font(.headline).foregroundStyle(.black).frame(maxWidth: .infinity).padding(.vertical, 16)
            .background(WFStyle.gold).clipShape(RoundedRectangle(cornerRadius: 12))
        }
      }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 32)
    }.scrollDismissesKeyboard(.interactively).navigationBarHidden(true)
      .onAppear {
        name = storedName; platform = storedPlatform
      }
      .alert(t("settingsSaved"), isPresented: $saved) { Button(t("ok"), role: .cancel) {} }
  }
}

private struct SettingsIcon: View {
  let name: String
  var body: some View {
    Image(systemName: name).font(.system(size: 17, weight: .semibold)).foregroundStyle(WFStyle.accent)
      .frame(width: 34, height: 34).background(WFStyle.accent.opacity(0.1)).clipShape(RoundedRectangle(cornerRadius: 9))
  }
}

private struct SettingsRow: View {
  let icon: String
  let title: String
  let value: String
  var body: some View {
    HStack(spacing: 12) {
      SettingsIcon(name: icon)
      VStack(alignment: .leading, spacing: 4) {
        Text(t(title)).font(.subheadline).foregroundStyle(Color(uiColor: .secondaryLabel))
        Text(td(value)).font(.body.weight(.semibold)).foregroundStyle(Color(uiColor: .label)).lineLimit(1)
      }
      Spacer(minLength: 8)
      Image(systemName: "chevron.up.chevron.down").font(.caption.weight(.semibold)).foregroundStyle(Color(uiColor: .secondaryLabel))
    }.padding(16).contentShape(Rectangle())
  }
}

private struct SettingsLinkRow: View {
  let icon: String
  let title: String

  var body: some View {
    HStack(spacing: 12) {
      SettingsIcon(name: icon)
      Text(t(title)).font(.body.weight(.semibold)).foregroundStyle(Color(uiColor: .label))
      Spacer(minLength: 8)
      Image(systemName: "chevron.right").font(.caption.weight(.semibold)).foregroundStyle(Color(uiColor: .secondaryLabel))
    }.padding(16).contentShape(Rectangle())
  }
}

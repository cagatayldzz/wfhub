import SwiftUI

func screenDate(_ text: String) -> Date? {
  let formatter = ISO8601DateFormatter()
  formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
  if let date = formatter.date(from: text) { return date }
  formatter.formatOptions = [.withInternetDateTime]
  return formatter.date(from: text)
}

private func imageURL(_ value: Any?) -> URL? {
  guard let string = value as? String, !string.isEmpty else { return nil }
  if let url = URL(string: string), url.scheme != nil { return url }
  return URL(string: "https://raw.githubusercontent.com/WFCD/warframe-items/master/data/img/" + string.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed)!)
}

struct CodexRow: View {
  let item: [String: Any]
  var body: some View {
    HStack(spacing: 12) {
      AsyncImage(url: imageURL(item["imageName"])) { image in image.resizable().scaledToFit() } placeholder: { Color.secondary.opacity(0.1) }
        .frame(width: 64, height: 64).clipped()
      Text(itemTitle(item)).font(.headline)
    }.padding(.vertical, 4)
  }
}

struct CodexTile: View {
  let item: [String: Any]
  var body: some View { ArtworkCard(title: itemTitle(item), image: imageURL(item["imageName"])) }
}

struct CodexDetail: View {
  let item: [String: Any]
  private let stats = [("health", "Health"), ("shield", "Shield"), ("armor", "Armor"), ("stamina", "Stamina"), ("power", "Power"), ("sprintSpeed", "Sprint Speed"), ("masteryReq", "Mastery Requirement")]
  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 14) {
        VStack(spacing: 10) {
          AsyncImage(url: imageURL(item["imageName"])) { image in image.resizable().scaledToFit() } placeholder: { Color.clear }
            .frame(height: 190).frame(maxWidth: .infinity)
          Text(itemTitle(item)).font(.title2.bold()).foregroundStyle(.white).multilineTextAlignment(.center)
        }.padding(20).frame(maxWidth: .infinity).background(WFStyle.artwork).clipShape(RoundedRectangle(cornerRadius: 12))
        if let description = item["description"] as? String, !description.isEmpty {
          WFCard { VStack(alignment: .leading, spacing: 10) {
            Text("Description").font(.headline)
            Text(description).font(.subheadline).foregroundStyle(.secondary).textSelection(.enabled)
          } }
        }
        if stats.contains(where: { item[$0.0] != nil }) {
          WFCard { VStack(alignment: .leading, spacing: 12) {
            Text("Information").font(.headline)
            ForEach(stats, id: \.0) { key, title in
              if let value = item[key] {
                HStack { Text(title).foregroundStyle(.secondary); Spacer(); Text(String(describing: value)).fontWeight(.semibold) }.font(.subheadline)
              }
            }
          } }
        }
        if let abilities = item["abilities"] as? [[String: Any]], !abilities.isEmpty {
          WFCard { VStack(alignment: .leading, spacing: 14) {
            Text("Abilities").font(.headline)
            ForEach(Array(abilities.enumerated()), id: \.offset) { _, ability in
              HStack(alignment: .top, spacing: 12) {
                AsyncImage(url: imageURL(ability["imageName"])) { image in image.resizable().scaledToFit() } placeholder: { Color.clear }
                  .frame(width: 48, height: 48).background(WFStyle.artwork).clipShape(RoundedRectangle(cornerRadius: 7))
                VStack(alignment: .leading, spacing: 4) {
                  Text(ability["name"] as? String ?? "Ability").font(.subheadline.bold())
                  if let description = ability["description"] as? String { Text(description).font(.subheadline).foregroundStyle(.secondary) }
                }
              }
            }
          } }
        }
        if let damage = item["damage"] as? [String: Any], !damage.isEmpty {
          WFCard { VStack(alignment: .leading, spacing: 10) { Text("Damage").font(.headline); DetailFields(value: damage, depth: 0) } }
        }
      }.padding(12)
    }
  }
}

struct ActivityRow: View {
  let value: [String: Any]
  let endpoint: String

  private var primary: String {
    if endpoint == "alerts", let mission = value["mission"] as? [String: Any] { return mission["node"] as? String ?? "Alert" }
    if endpoint == "sortie" || endpoint == "archonHunt" { return value["boss"] as? String ?? "Mission" }
    if endpoint == "nightwave" { return "Nightwave" }
    if endpoint == "voidTrader" { return value["character"] as? String ?? "Baro Ki'Teer" }
    return itemTitle(value)
  }
  private var detail: String? {
    for key in ["missionType", "faction", "location", "state", "desc", "modifier", "description"] {
      if let text = value[key] as? String, !text.isEmpty, text != primary { return text }
    }
    return nil
  }
  var body: some View {
    WFCard {
      VStack(alignment: .leading, spacing: 10) {
        HStack(alignment: .top) {
          Text(primary).font(.headline).fixedSize(horizontal: false, vertical: true)
          Spacer(minLength: 8)
          if let tier = value["tier"] as? String { Text(tier.uppercased()).font(.caption.bold()).foregroundStyle(WFStyle.accent) }
        }
        if let detail { Text(detail).font(.subheadline).foregroundStyle(.secondary) }
        if let expiry = value["expiry"] as? String, let date = screenDate(expiry), date > .now {
          Label { Text(date, style: .timer).monospacedDigit() } icon: { Image(systemName: "clock") }.font(.caption.bold()).foregroundStyle(.secondary)
        }
        ForEach(["node", "enemy", "faction", "target", "item", "discount", "price", "fomorianProgress", "razorbackProgress"], id: \.self) { key in
          if let field = value[key], !(field is NSNull) {
            let shown = String(describing: field)
            if shown != primary && shown != detail {
              HStack {
                Text(key.replacingOccurrences(of: "([a-z])([A-Z])", with: "$1 $2", options: .regularExpression).capitalized).foregroundStyle(.secondary)
                Spacer()
                Text(shown).fontWeight(.medium).multilineTextAlignment(.trailing)
              }.font(.subheadline)
            }
          }
        }
        if let variants = value["variants"] as? [[String: Any]], !variants.isEmpty {
          ForEach(Array(variants.enumerated()), id: \.offset) { index, variant in
            Divider()
            HStack(alignment: .top, spacing: 10) {
              Text("\(index + 1)").font(.caption.bold()).foregroundStyle(.black).padding(7).background(WFStyle.gold, in: Circle())
              VStack(alignment: .leading, spacing: 4) {
                Text(variant["node"] as? String ?? variant["nodeKey"] as? String ?? "Mission").font(.subheadline.bold())
                Text(variant["modifier"] as? String ?? variant["missionType"] as? String ?? "").font(.caption).foregroundStyle(.secondary)
              }
            }
          }
        }
        if let challenges = value["activeChallenges"] as? [[String: Any]] {
          ForEach(Array(challenges.enumerated()), id: \.offset) { _, challenge in
            Divider()
            HStack(alignment: .top) {
              VStack(alignment: .leading, spacing: 4) {
                Text(challenge["title"] as? String ?? "Challenge").font(.subheadline.bold())
                Text(challenge["desc"] as? String ?? "").font(.caption).foregroundStyle(.secondary)
              }
              Spacer()
              if let points = challenge["reputation"] as? Int { Text("\(points)").font(.caption.bold()).foregroundStyle(WFStyle.accent) }
            }
          }
        }
        if let inventory = value["inventory"] as? [[String: Any]], !inventory.isEmpty {
          ForEach(Array(inventory.enumerated()), id: \.offset) { _, entry in
            Divider()
            HStack { Text(entry["item"] as? String ?? "Item").font(.subheadline); Spacer(); Text("\(entry["ducats"] as? Int ?? 0) ◈").font(.caption.bold()).foregroundStyle(WFStyle.accent) }
          }
        }
        if let mission = value["mission"] as? [String: Any] {
          if let type = mission["type"] as? String { Text(type).font(.subheadline).foregroundStyle(.secondary) }
          if let reward = mission["reward"] as? [String: Any], let items = reward["items"] as? [String], !items.isEmpty { Text(items.joined(separator: ", ")).font(.subheadline) }
        }
      }
    }
  }
}

struct DetailFields: View {
  let value: [String: Any]
  let depth: Int
  var body: some View {
    ForEach(value.keys.sorted(), id: \.self) { key in
      if let text = value[key] as? String, !text.isEmpty, key != "link", !key.hasSuffix("Key"), !key.hasSuffix("Id") {
        LabeledContent(label(key), value: text).font(.subheadline)
      } else if let number = value[key] as? NSNumber {
        LabeledContent(label(key), value: number.stringValue).font(.subheadline)
      } else if let object = value[key] as? [String: Any], !object.isEmpty, depth < 3 {
        VStack(alignment: .leading, spacing: 5) {
          Text(label(key)).font(.subheadline.bold())
          DetailFields(value: object, depth: depth + 1)
        }
      } else if let strings = value[key] as? [String], !strings.isEmpty {
        LabeledContent(label(key), value: strings.joined(separator: ", ")).font(.subheadline)
      } else if let objects = value[key] as? [[String: Any]], !objects.isEmpty, depth < 3 {
        VStack(alignment: .leading, spacing: 8) {
          Text(label(key)).font(.subheadline.bold())
          ForEach(Array(objects.enumerated()), id: \.offset) { _, object in
            DetailFields(value: object, depth: depth + 1).padding(.leading, 8)
          }
        }
      }
    }
  }
  private func label(_ key: String) -> String {
    key.replacingOccurrences(of: "([a-z])([A-Z])", with: "$1 $2", options: .regularExpression).capitalized
  }
}

struct NewsDetailRow: View {
  let item: [String: Any]
  @Environment(\.locale) private var locale
  private var title: String {
    let translations = item["translations"] as? [String: String] ?? [:]
    let language = locale.language.languageCode?.identifier ?? "en"
    return translations[language] ?? translations["en"] ?? item["message"] as? String ?? "News"
  }
  var body: some View {
    let content = VStack(alignment: .leading, spacing: 8) {
      if let image = item["imageLink"] as? String {
        AsyncImage(url: URL(string: image)) { image in image.resizable().scaledToFill() } placeholder: { Color.secondary.opacity(0.1) }
          .frame(height: 180).frame(maxWidth: .infinity).clipped().cornerRadius(10)
      }
      Text(title.uppercased()).font(.headline)
      if let value = item["date"] as? String, let date = screenDate(value) {
        Text(date, format: .dateTime.day().month().year().hour().minute()).foregroundStyle(.secondary)
      }
    }.padding(.vertical, 6)
    if let link = item["link"] as? String, let url = URL(string: link), ["http", "https"].contains(url.scheme?.lowercased() ?? "") {
      Link(destination: url) { content }.buttonStyle(.plain)
    } else { content }
  }
}

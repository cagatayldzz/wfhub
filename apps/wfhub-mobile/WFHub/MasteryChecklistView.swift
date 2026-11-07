import SwiftUI

private struct MasteryWarframe: Identifiable, Codable {
  let name: String
  let image: String?
  let level: Int?
  let mastery: Int
  let group: String?
  private let sourceID: String?
  var slug: String { name.folding(options: [.diacriticInsensitive, .widthInsensitive], locale: Locale(identifier: "en_US_POSIX")).lowercased().replacingOccurrences(of: "[^a-z0-9]+", with: "-", options: .regularExpression).trimmingCharacters(in: CharacterSet(charactersIn: "-")) }
  var id: String { sourceID ?? slug }
  enum CodingKeys: String, CodingKey { case name, image, level, mastery, group; case sourceID = "id" }
}

private struct MasteryMark: Codable {
  var completed: Bool
  var modified: TimeInterval
}

@MainActor private final class MasteryStore: ObservableObject {
  @Published private(set) var marks: [String: MasteryMark] = [:]
  private let localKey = "mastery.warframes.marks.v1"
  private let prefix = "mastery.warframe.v1."
  private let cloud = NSUbiquitousKeyValueStore.default
  private let groupPrefix = "mastery.group.v1."
  private let legacyCategories: Set<String> = ["warframes", "primary", "secondary", "melee"]
  private var observer: NSObjectProtocol?

  init() {
    if let data = UserDefaults.standard.data(forKey: localKey),
       let saved = try? JSONDecoder().decode([String: MasteryMark].self, from: data) { marks = saved }
    observer = NotificationCenter.default.addObserver(forName: NSUbiquitousKeyValueStore.didChangeExternallyNotification, object: cloud, queue: .main) { [weak self] _ in
      Task { @MainActor in self?.mergeCloud() }
    }
    cloud.synchronize()
    mergeCloud()
  }

  deinit { if let observer { NotificationCenter.default.removeObserver(observer) } }

  func toggle(_ id: String) {
    let mark = MasteryMark(completed: !(marks[id]?.completed ?? false), modified: Date().timeIntervalSince1970)
    marks[id] = mark
    saveLocal()
    writeCloud(mark, id: id)
    cloud.synchronize()
  }

  private func mergeCloud() {
    for (key, value) in cloud.dictionaryRepresentation where key.hasPrefix(groupPrefix) {
      guard let data = value as? Data, let remote = try? JSONDecoder().decode([String: MasteryMark].self, from: data) else { continue }
      for (id, mark) in remote where marks[id] == nil || marks[id]!.modified < mark.modified { marks[id] = mark }
    }
    for (key, value) in cloud.dictionaryRepresentation where key.hasPrefix(prefix) {
      guard let data = value as? Data, let remote = try? JSONDecoder().decode(MasteryMark.self, from: data) else { continue }
      let id = String(key.dropFirst(prefix.count))
      if let local = marks[id], local.modified >= remote.modified { continue }
      marks[id] = remote
    }
    saveLocal()
    let groups = Set(marks.keys.compactMap { group(for: $0) })
    for group in groups { writeGroup(group) }
    for (id, mark) in marks where group(for: id) == nil {
      let remote = cloud.data(forKey: prefix + id).flatMap { try? JSONDecoder().decode(MasteryMark.self, from: $0) }
      if remote == nil || mark.modified > remote!.modified { writeCloud(mark, id: id) }
    }
  }

  private func saveLocal() {
    if let data = try? JSONEncoder().encode(marks) { UserDefaults.standard.set(data, forKey: localKey) }
  }

  private func group(for id: String) -> String? {
    guard let separator = id.firstIndex(of: ":") else { return nil }
    let category = String(id[..<separator])
    return legacyCategories.contains(category) ? nil : category
  }

  private func writeGroup(_ group: String) {
    let local = marks.filter { self.group(for: $0.key) == group }
    let remote = cloud.data(forKey: groupPrefix + group).flatMap { try? JSONDecoder().decode([String: MasteryMark].self, from: $0) } ?? [:]
    var merged = remote
    for (id, mark) in local where merged[id] == nil || merged[id]!.modified < mark.modified { merged[id] = mark }
    if let data = try? JSONEncoder().encode(merged) { cloud.set(data, forKey: groupPrefix + group) }
  }

  private func writeCloud(_ mark: MasteryMark, id: String) {
    if let group = group(for: id) { writeGroup(group) }
    else if let data = try? JSONEncoder().encode(mark) { cloud.set(data, forKey: prefix + id) }
  }
}

struct MasteryChecklistView: View {
  @StateObject private var store = MasteryStore()
  @AppStorage("mastery.snapshot.updated") private var updatedAt = ""
  private let categories = ["warframes", "primary", "secondary", "melee", "kitgun", "zaw", "amp", "sentinel", "moa", "hound", "beast", "robotic", "archwing", "archgun", "archmelee", "necramech", "railjack", "k-drive", "railjack-intrinsic", "drifter-intrinsic", "starchart", "steel-path", "founders", "china-founders"]

  var body: some View {
    ScrollView {
      VStack(spacing: 12) {
        if !updatedAt.isEmpty {
          HStack { Text("Checklist updated"); Spacer(); Text(updatedAt).monospacedDigit() }
            .font(.caption).foregroundStyle(.secondary).padding(.horizontal, 4)
        }
        ForEach(categories, id: \.self) { category in
          MasteryCategoryCard(category: category, store: store)
        }
      }.padding(12)
    }.navigationTitle("Mastery Rank Checklist").navigationBarTitleDisplayMode(.inline)
      .task {
        let url = URL(string: "https://wfhub.cagatayldzz.com/api/mastery/updated.json")!
        if let metadata = try? await API.fetch(url) as? [String: Any], let date = metadata["updatedAt"] as? String { updatedAt = date }
      }
  }
}

private struct MasteryCategoryCard: View {
  let category: String
  @ObservedObject var store: MasteryStore
  @State private var items: [MasteryWarframe] = []
  @State private var failed = false

  var body: some View {
    NavigationLink {
      MasteryCategoryList(category: category, store: store, items: $items)
    } label: {
      VStack(alignment: .leading, spacing: 12) {
        HStack {
          Text(MasteryCategoryList.title(for: category)).font(.headline)
          Spacer()
          Image(systemName: "chevron.right").font(.caption.bold()).foregroundStyle(.secondary)
        }
        if items.isEmpty {
          if failed { Text("Unable to load").font(.subheadline).foregroundStyle(.secondary) }
          else { ProgressView() }
        } else {
          let completed = items.filter { store.marks[MasteryCategoryList.markID($0, category: category)]?.completed == true }
          HStack {
            Text("\(completed.count) / \(items.count)").font(.subheadline.weight(.medium).monospacedDigit()).foregroundStyle(.secondary)
            Spacer()
            Text("\(completed.reduce(0) { $0 + MasteryCategoryList.mastery($1, category: category) }, format: .number) mastery")
              .font(.subheadline.weight(.semibold)).foregroundStyle(WFStyle.accent)
          }
          ProgressView(value: Double(completed.count), total: Double(items.count)).tint(WFStyle.accent)
        }
      }.padding(16).frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(uiColor: .secondarySystemBackground), in: RoundedRectangle(cornerRadius: 14))
    }.buttonStyle(.plain).disabled(items.isEmpty).task { await load() }
  }

  private func load() async {
    let cacheKey = "mastery.\(category).checklist.v2"
    if let data = UserDefaults.standard.data(forKey: cacheKey), let cached = try? JSONDecoder().decode([MasteryWarframe].self, from: data) { items = cached }
    do {
      let url = URL(string: "https://wfhub.cagatayldzz.com/api/mastery/\(category).json")!
      guard let raw = try await API.fetch(url) as? [[String: Any]] else { throw URLError(.cannotParseResponse) }
      let data = try JSONSerialization.data(withJSONObject: raw)
      let result = try JSONDecoder().decode([MasteryWarframe].self, from: data)
      guard !result.isEmpty else { throw URLError(.cannotParseResponse) }
      guard !Task.isCancelled else { return }
      items = result
      UserDefaults.standard.set(data, forKey: cacheKey)
      failed = false
    } catch { if !Task.isCancelled { failed = items.isEmpty } }
  }

}

private struct MasteryCategoryList: View {
  let category: String
  @ObservedObject var store: MasteryStore
  @Binding var items: [MasteryWarframe]
  @State private var search = ""
  @State private var filter = 0
  static func title(for category: String) -> LocalizedStringKey {
    let names: [String: String] = ["warframes": "Warframes", "primary": "Primary", "secondary": "Secondary", "melee": "Melee", "kitgun": "Kitgun", "zaw": "Zaw", "amp": "Amp", "sentinel": "Sentinel", "moa": "MOA", "hound": "Hound", "beast": "Beast", "robotic": "Robotic", "archwing": "Archwing", "archgun": "Archgun", "archmelee": "Archmelee", "necramech": "Necramech", "railjack": "Railjack", "k-drive": "K-Drive", "railjack-intrinsic": "Railjack Intrinsic", "drifter-intrinsic": "Drifter Intrinsic", "starchart": "Starchart", "steel-path": "The Steel Path", "founders": "Founders Pack", "china-founders": "China Founders Pack"]
    return LocalizedStringKey(names[category] ?? category)
  }
  static func markID(_ item: MasteryWarframe, category: String) -> String { category == "warframes" ? item.slug : "\(category):\(item.id)" }
  static func mastery(_ item: MasteryWarframe, category: String) -> Int { item.mastery }
  private var visible: [MasteryWarframe] {
    items.filter { item in
      let done = store.marks[Self.markID(item, category: category)]?.completed == true
      return (search.isEmpty || item.name.localizedCaseInsensitiveContains(search)) && (filter == 0 || (filter == 1 ? !done : done))
    }
  }

  private var groupNames: [String] {
    var seen = Set<String>()
    return items.compactMap(\.group).filter { seen.insert($0).inserted }
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 16) {
        Picker("Filter", selection: $filter) {
          Text("All").tag(0)
          Text("Remaining").tag(1)
          Text("Completed").tag(2)
        }.pickerStyle(.segmented)
        if items.isEmpty {
          ProgressView().frame(maxWidth: .infinity).padding()
        } else if visible.isEmpty {
          ContentUnavailableView("No items found", systemImage: "magnifyingglass")
        } else {
          LazyVStack(alignment: .leading, spacing: 16) {
            if groupNames.isEmpty {
              itemRows(visible)
            } else {
              ForEach(groupNames, id: \.self) { group in
                let allItems = items.filter { $0.group == group }
                let matching = visible.filter { $0.group == group }
                if !matching.isEmpty {
                  Section {
                    itemRows(matching)
                  } header: {
                    let completed = allItems.filter { store.marks[Self.markID($0, category: category)]?.completed == true }.count
                    HStack {
                      Text(LocalizedStringKey(group)).font(.headline)
                      Spacer()
                      Text("\(completed) / \(allItems.count)").font(.subheadline.monospacedDigit()).foregroundStyle(.secondary)
                    }.padding(.horizontal, 4).padding(.bottom, 2)
                  }
                }
              }
              let ungrouped = visible.filter { $0.group == nil }
              if !ungrouped.isEmpty { itemRows(ungrouped) }
            }
          }

        }
      }.padding(12)
    }.navigationTitle(Self.title(for: category)).navigationBarTitleDisplayMode(.inline)
      .searchable(text: $search)
  }

  private func itemRows(_ rows: [MasteryWarframe]) -> some View {
    ForEach(rows) { item in
      let done = store.marks[Self.markID(item, category: category)]?.completed == true
      Button { store.toggle(Self.markID(item, category: category)) } label: {
        HStack(spacing: 12) {
          if let image = item.image, let url = URL(string: image) {
            ThumbnailImage(url: url, maxPixelSize: 160).frame(width: 52, height: 52)
          }
          VStack(alignment: .leading, spacing: 4) {
            Text(item.name).font(.headline).foregroundStyle(.primary)
            if let level = item.level { Text("Rank \(level) · \(item.mastery, format: .number) mastery").font(.caption).foregroundStyle(.secondary) }
            else { Text("\(item.mastery, format: .number) mastery").font(.caption).foregroundStyle(.secondary) }
          }
          Spacer()
          Image(systemName: done ? "checkmark.circle.fill" : "circle")
            .font(.title2).foregroundStyle(done ? WFStyle.accent : Color.secondary)
        }.padding(12).frame(maxWidth: .infinity, alignment: .leading)
          .background(Color(uiColor: .secondarySystemBackground), in: RoundedRectangle(cornerRadius: 12))
      }.buttonStyle(.plain).accessibilityLabel(item.name).accessibilityValue(done ? "Completed" : "Not completed")
    }
  }

}

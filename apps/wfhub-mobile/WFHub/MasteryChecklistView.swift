import SwiftUI

private struct MasteryWarframe: Identifiable, Codable {
  let name: String
  let slug: String
  let imageName: String?
  var id: String { slug }
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
    for (key, value) in cloud.dictionaryRepresentation where key.hasPrefix(prefix) {
      guard let data = value as? Data, let remote = try? JSONDecoder().decode(MasteryMark.self, from: data) else { continue }
      let id = String(key.dropFirst(prefix.count))
      if let local = marks[id], local.modified >= remote.modified { continue }
      marks[id] = remote
    }
    saveLocal()
    for (id, mark) in marks {
      let remote = cloud.data(forKey: prefix + id).flatMap { try? JSONDecoder().decode(MasteryMark.self, from: $0) }
      if remote == nil || mark.modified > remote!.modified { writeCloud(mark, id: id) }
    }
  }

  private func saveLocal() {
    if let data = try? JSONEncoder().encode(marks) { UserDefaults.standard.set(data, forKey: localKey) }
  }

  private func writeCloud(_ mark: MasteryMark, id: String) {
    if let data = try? JSONEncoder().encode(mark) { cloud.set(data, forKey: prefix + id) }
  }
}

struct MasteryChecklistView: View {
  @StateObject private var store = MasteryStore()

  var body: some View {
    ScrollView {
      VStack(spacing: 12) {
        ForEach(["warframes", "primary", "secondary", "melee"], id: \.self) { category in
          MasteryCategoryCard(category: category, store: store)
        }
      }.padding(12)
    }.navigationTitle("Mastery Rank Checklist").navigationBarTitleDisplayMode(.inline)
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
    let cacheKey = "mastery.\(category).catalogue.v1"
    if let data = UserDefaults.standard.data(forKey: cacheKey), let cached = try? JSONDecoder().decode([MasteryWarframe].self, from: data) { items = cached }
    do {
      let url = URL(string: "https://wfhub.cagatayldzz.com/api/\(category).json")!
      guard let raw = try await API.fetch(url) as? [[String: Any]] else { throw URLError(.cannotParseResponse) }
      let result = raw.compactMap { entry -> MasteryWarframe? in
        guard let name = entry["name"] as? String, let slug = entry["slug"] as? String else { return nil }
        if category == "warframes" && MasteryCategoryList.excluded.contains(slug) { return nil }
        if category == "melee" && MasteryCategoryList.meleeParts.contains(slug) { return nil }
        if category == "primary" && ["sirocco", "purgator-1"].contains(slug) { return nil }
        return MasteryWarframe(name: name, slug: slug, imageName: entry["imageName"] as? String)
      }.sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
      guard !result.isEmpty else { throw URLError(.cannotParseResponse) }
      guard !Task.isCancelled else { return }
      items = result
      if let data = try? JSONEncoder().encode(result) { UserDefaults.standard.set(data, forKey: cacheKey) }
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
    switch category {
    case "primary": return "Primary"
    case "secondary": return "Secondary"
    case "melee": return "Melee"
    default: return "Warframes"
    }
  }
  static func markID(_ item: MasteryWarframe, category: String) -> String { category == "warframes" ? item.id : "\(category):\(item.id)" }
  static func maxRank(_ item: MasteryWarframe, category: String) -> Int {
    let name = item.name.lowercased()
    return category != "warframes" && (name.hasPrefix("kuva ") || name.hasPrefix("tenet ") || name.hasPrefix("coda ") || name.hasPrefix("dual coda ") || name == "paracesis") ? 40 : 30
  }
  static func mastery(_ item: MasteryWarframe, category: String) -> Int { category == "warframes" ? 6000 : maxRank(item, category: category) * 100 }
  static let meleeParts: Set<String> = ["balla", "cyath", "dehtat", "dokrahm", "kronsh", "mewan", "ooltha", "plague-keewar", "plague-kripath", "rabvee", "sepfahn", "ekwana-ii-jai", "ekwana-ii-ruhang", "ekwana-jai", "ekwana-jai-ii", "ekwana-ruhang", "ekwana-ruhang-ii", "jai", "jai-ii", "jayap", "korb", "kwath", "laka", "peye", "plague-akwin", "plague-bokwin", "ruhang", "ruhang-ii", "seekalla", "shtung", "vargeet-ii-jai", "vargeet-ii-ruhang", "vargeet-jai", "vargeet-jai-ii", "vargeet-ruhang", "vargeet-ruhang-ii"]
  // The upstream Warframes category also contains Necramechs, Helminth and an alternate form.
  static let excluded: Set<String> = ["bonewidow", "voidrig", "helminth", "orion-sirius"]
  private var visible: [MasteryWarframe] {
    items.filter { item in
      let done = store.marks[Self.markID(item, category: category)]?.completed == true
      return (search.isEmpty || item.name.localizedCaseInsensitiveContains(search)) && (filter == 0 || (filter == 1 ? !done : done))
    }
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
          LazyVStack(spacing: 8) {
            ForEach(visible) { item in
              let done = store.marks[Self.markID(item, category: category)]?.completed == true
              Button { store.toggle(Self.markID(item, category: category)) } label: {
                HStack(spacing: 12) {
                  ThumbnailImage(url: item.imageName.flatMap(URL.init(string:)), maxPixelSize: 160).frame(width: 52, height: 52)
                  VStack(alignment: .leading, spacing: 4) {
                    Text(item.name).font(.headline).foregroundStyle(.primary)
                    Text("Rank \(Self.maxRank(item, category: category)) · \(Self.mastery(item, category: category), format: .number) mastery").font(.caption).foregroundStyle(.secondary)
                  }
                  Spacer()
                  Image(systemName: done ? "checkmark.circle.fill" : "circle")
                    .font(.title2).foregroundStyle(done ? WFStyle.accent : Color.secondary)
                }.padding(12).background(Color(uiColor: .secondarySystemBackground), in: RoundedRectangle(cornerRadius: 12))
              }.buttonStyle(.plain).accessibilityLabel(item.name).accessibilityValue(done ? "Completed" : "Not completed")
            }
          }
        }
      }.padding(12)
    }.navigationTitle(Self.title(for: category)).navigationBarTitleDisplayMode(.inline)
      .searchable(text: $search)
  }
}

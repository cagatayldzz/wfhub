import SwiftUI

struct FeedbackView: View {
  @Environment(\.dismiss) private var dismiss
  @State private var title = ""
  @State private var category = "bug"
  @State private var message = ""
  @State private var sending = false
  @State private var sent = false
  @State private var error: String?
  @FocusState private var focused: Bool

  private var valid: Bool {
    (3...120).contains(title.trimmingCharacters(in: .whitespacesAndNewlines).unicodeScalars.count) &&
      (10...4000).contains(message.trimmingCharacters(in: .whitespacesAndNewlines).unicodeScalars.count)
  }

  var body: some View {
    ScrollView {
      VStack(alignment: .leading, spacing: 24) {
        Text("Feedback").font(.system(size: 28, weight: .heavy))
        Text("Help us improve WFHub. Please don't include personal information.")
          .font(.subheadline).foregroundStyle(.secondary)
        VStack(alignment: .leading, spacing: 10) {
          Text("CATEGORY").font(.caption.bold()).tracking(1).foregroundStyle(.secondary)
          Picker("Category", selection: $category) {
            Text("Bug report").tag("bug")
            Text("Suggestion").tag("suggestion")
            Text("Other").tag("other")
          }.pickerStyle(.segmented)
        }
        VStack(alignment: .leading, spacing: 10) {
          Text("TITLE").font(.caption.bold()).tracking(1).foregroundStyle(.secondary)
          TextField("Briefly describe your feedback", text: $title)
            .focused($focused).padding(16)
            .background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
          Text("3–120 characters").font(.caption).foregroundStyle(.secondary)
        }
        VStack(alignment: .leading, spacing: 10) {
          Text("MESSAGE").font(.caption.bold()).tracking(1).foregroundStyle(.secondary)
          TextEditor(text: $message).focused($focused).frame(minHeight: 160)
            .scrollContentBackground(.hidden).padding(12)
            .background(Color(uiColor: .secondarySystemBackground)).clipShape(RoundedRectangle(cornerRadius: 14))
          Text("10–4000 characters").font(.caption).foregroundStyle(.secondary)
        }
        if let error { Text(LocalizedStringKey(error)).font(.subheadline).foregroundStyle(.red) }
        Button { Task { await submit() } } label: {
          Group { if sending { ProgressView().tint(.black) } else { Text("Send feedback").font(.headline) } }
            .foregroundStyle(.black).frame(maxWidth: .infinity).padding(.vertical, 16)
            .background(WFStyle.gold).clipShape(RoundedRectangle(cornerRadius: 12))
        }.disabled(!valid || sending).opacity(valid && !sending ? 1 : 0.55)
      }.padding(.horizontal, 16).padding(.top, 24).padding(.bottom, 32)
    }
    .scrollDismissesKeyboard(.interactively)
    .navigationBarTitleDisplayMode(.inline)
    .alert("Thank you for your feedback!", isPresented: $sent) {
      Button("OK") { dismiss() }
    }
  }

  private func submit() async {
    sending = true
    focused = false
    error = nil
    defer { sending = false }
    do {
      var request = URLRequest(url: URL(string: "https://wfhub.cagatayldzz.com/api/feedback")!)
      request.httpMethod = "POST"
      request.setValue("application/json", forHTTPHeaderField: "Content-Type")
      request.httpBody = try JSONEncoder().encode(Submission(title: title, category: category, message: message))
      let (_, response) = try await URLSession.shared.data(for: request)
      guard let http = response as? HTTPURLResponse else { throw URLError(.badServerResponse) }
      if http.statusCode == 429 { error = "Too many submissions. Please try again later."; return }
      guard http.statusCode == 201 else { throw URLError(.badServerResponse) }
      sent = true
    } catch {
      self.error = "Couldn't send feedback. Please try again."
    }
  }

  private struct Submission: Encodable {
    let title: String
    let category: String
    let message: String
  }
}

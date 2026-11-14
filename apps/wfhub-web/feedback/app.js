(async () => {
  const meta = document.getElementById("meta");
  const list = document.getElementById("list");
  try {
    const response = await fetch("/api/admin/feedback", { credentials: "same-origin", cache: "no-store" });
    if (!response.ok) throw new Error("HTTP " + response.status);
    const items = await response.json();
    meta.textContent = items.length + " item" + (items.length === 1 ? "" : "s");
    for (const item of items) {
      const card = document.createElement("div");
      card.className = "item";
      const row = document.createElement("div");
      row.className = "row";
      const title = Object.assign(document.createElement("span"), { className: "title", textContent: item.title });
      const tag = Object.assign(document.createElement("span"), { className: "tag " + String(item.category).replace(/[^a-z]/g, ""), textContent: item.category });
      const date = Object.assign(document.createElement("span"), { className: "date", textContent: new Date(item.created_at).toLocaleString() });
      row.append(title, tag, date);
      const message = Object.assign(document.createElement("div"), { className: "msg", textContent: item.message });
      card.append(row, message);
      list.append(card);
    }
  } catch (error) {
    meta.textContent = "Could not load feedback (" + error.message + ")";
  }
})();

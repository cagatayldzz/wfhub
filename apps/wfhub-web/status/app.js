(async () => {
  const banner = document.getElementById("banner");
  const summary = document.getElementById("summary");
  const list = document.getElementById("services");
  const DAYS = 90;

  const el = (tag, className, text) => {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  };

  function ago(seconds) {
    const minutes = Math.max(0, Math.round((Date.now() / 1000 - seconds) / 60));
    if (minutes < 1) return "just now";
    if (minutes < 60) return minutes + " min ago";
    return Math.round(minutes / 60) + " h ago";
  }

  function bars(days) {
    const byDate = new Map(days.map((day) => [day.date, day.uptime]));
    const wrap = el("div", "bars");
    for (let i = DAYS - 1; i >= 0; i--) {
      const date = new Date(Date.now() - i * 86400000).toISOString().slice(0, 10);
      const uptime = byDate.get(date);
      const bar = el("div", "bar" + (uptime === undefined ? "" : uptime >= 99 ? " ok" : uptime >= 90 ? " warn" : " bad"));
      bar.title = date + (uptime === undefined ? ": no data" : ": " + uptime + "% uptime");
      wrap.append(bar);
    }
    return wrap;
  }

  try {
    const response = await fetch("/api/status", { cache: "no-store" });
    if (!response.ok) throw new Error("HTTP " + response.status);
    const { services } = await response.json();
    for (const service of services) {
      const card = el("section", "service");
      const row = el("div", "row");
      const state = el("span", "state " + (service.status || ""));
      const dot = el("span", "dot");
      state.append(dot, service.status === "up" ? "Operational" : service.status === "down" ? "Down" : "No data yet");
      row.append(el("h2", "", service.name), state);
      const scale = el("div", "scale");
      scale.append(el("span", "", DAYS + " days ago"), el("span", "", service.uptime === null ? "" : service.uptime + "% uptime"), el("span", "", "Today"));
      card.append(row, bars(service.days), scale);
      if (service.status) {
        const parts = ["Last check " + ago(service.checkedAt), service.latencyMs + " ms"];
        if (service.status === "down" && service.detail) parts.push(service.detail);
        card.append(el("p", "detail", parts.join(" · ")));
      }
      list.append(card);
    }
    const known = services.filter((service) => service.status);
    const down = known.filter((service) => service.status === "down");
    banner.classList.add(!known.length ? "none" : down.length ? "down" : "up");
    summary.textContent = !known.length ? "No checks yet" : down.length ? down.map((service) => service.name).join(" and ") + (down.length > 1 ? " are" : " is") + " down" : "All systems operational";
  } catch (error) {
    banner.classList.add("down");
    summary.textContent = "Could not load the status (" + error.message + ")";
  }
})();

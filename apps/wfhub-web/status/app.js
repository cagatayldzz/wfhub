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

  const tip = el("div", "tip");
  tip.hidden = true;
  document.body.append(tip);
  const dateFormat = new Intl.DateTimeFormat("en-US", { month: "short", day: "2-digit", year: "numeric", timeZone: "UTC" });

  function showTip(bar) {
    const { kind, date, minutes } = bar.dataset;
    const title = { up: "Operational", down: "Downtime", none: "Not monitored" }[kind];
    tip.replaceChildren();
    const head = el("div", "tip-head");
    head.append(el("span", "dot " + kind), title);
    tip.append(head);
    if (kind === "down") tip.append(el("div", "tip-sub", "Down for " + minutes + " minute" + (minutes === "5" ? "" : "s")));
    if (kind === "none") tip.append(el("div", "tip-sub", "Not monitored"));
    tip.append(el("div", "tip-date", dateFormat.format(new Date(date + "T00:00:00Z"))));
    tip.hidden = false;
    const box = bar.getBoundingClientRect();
    const width = tip.offsetWidth;
    const left = Math.min(Math.max(8, box.left + box.width / 2 - width / 2), document.documentElement.clientWidth - width - 8);
    tip.style.left = left + window.scrollX + "px";
    tip.style.top = box.bottom + 14 + window.scrollY + "px";
  }
  const hideTip = () => { tip.hidden = true; };

  function bars(days) {
    const byDate = new Map(days.map((day) => [day.date, day]));
    const wrap = el("div", "bars");
    for (let i = DAYS - 1; i >= 0; i--) {
      const date = new Date(Date.now() - i * 86400000).toISOString().slice(0, 10);
      const day = byDate.get(date);
      const kind = !day ? "none" : day.downChecks > 0 ? "down" : "up";
      const bar = el("div", "bar " + kind);
      Object.assign(bar.dataset, { kind, date, minutes: day ? day.downChecks * 5 : 0 });
      bar.tabIndex = 0;
      bar.addEventListener("mouseenter", () => showTip(bar));
      bar.addEventListener("focus", () => showTip(bar));
      bar.addEventListener("click", () => showTip(bar));
      bar.addEventListener("mouseleave", hideTip);
      bar.addEventListener("blur", hideTip);
      wrap.append(bar);
    }
    return wrap;
  }
  document.addEventListener("scroll", hideTip, { passive: true });

  try {
    const response = await fetch("/api/status", { cache: "no-store" });
    if (!response.ok) throw new Error("HTTP " + response.status);
    const { services } = await response.json();
    for (const service of services) {
      const card = el("section", "service");
      const row = el("div", "row");
      const state = el("span", "state " + (service.status || ""));
      const dot = el("span", "dot");
      const compare = service.id === "compare";
      state.append(dot, service.status === "up" ? (compare ? "Matching" : "Operational") : service.status === "down" ? (compare ? "Differs" : "Down") : "No data yet");
      row.append(el("h2", "", service.name), state);
      const scale = el("div", "scale");
      scale.append(el("span", "", DAYS + " days ago"), el("span", "", service.uptime === null ? "" : service.uptime + "% uptime"), el("span", "", "Today"));
      card.append(row, bars(service.days), scale);
      if (service.status) {
        const parts = ["Last check " + ago(service.checkedAt)];
        if (!compare) parts.push(service.latencyMs + " ms");
        if (service.status === "down" && service.detail) parts.push(service.detail);
        card.append(el("p", "detail", parts.join(" · ")));
      }
      list.append(card);
    }
    const known = services.filter((service) => service.status);
    const down = known.filter((service) => service.status === "down");
    banner.classList.add(!known.length ? "none" : down.length ? "down" : "up");
    const broken = down.map((service) => (service.id === "compare" ? "The two APIs show different data" : service.name + " is down"));
    summary.textContent = !known.length ? "No checks yet" : down.length ? broken.join(" · ") : "All systems operational";
  } catch (error) {
    banner.classList.add("down");
    summary.textContent = "Could not load the status (" + error.message + ")";
  }
})();

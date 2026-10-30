const content = document.querySelector('#content');
const tabs = document.querySelector('#tabs');
const cdn = 'https://cdn.cagatayldzz.com/constants/wfhub/';
const icons = {
  home: '<path d="M3 11.5 12 4l9 7.5M5.5 10v9.5h13V10"/>',
  activities: '<path d="M13 3 5 13.5h6L10 21l8-10.5h-6z"/>',
  market: '<path d="M5 8h14l-1 12H6zM9 8V6a3 3 0 0 1 6 0v2"/>',
  codex: '<path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 21V5"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>'
};
const svg = (name, extra = '') => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" ${extra}>${icons[name]}</svg>`;
const pages = [['Home', 'home'], ['Activities', 'activities'], ['Market', 'market'], ['Codex', 'codex'], ['Settings', 'settings']];
const categories = ['arcanes','arch-gun','arch-melee','archwing','enemy','fish','gear','glyphs','melee','misc','mods','node','pets','primary','quests','railjack','relics','resources','secondary','sentinels','sentinelweapons','sigils','skins','warframes'];
const activities = [['Alerts','alerts'],["Baro Ki’Teer",'voidTrader'],['Fissures','fissures'],['Nightwave','nightwave'],['Events','events'],['Invasions','invasions'],['Arbitration','arbitration'],['Archon Hunt','archonHunt'],['Steel Path','steelPath'],['Sentient Outposts','sentientOutposts'],['Simaris','simaris'],['Construction Progress','constructionProgress'],['News','news'],['Market','flashSales'],['Sortie','sortie'],['Daily Deals','dailyDeals'],['Earth Cycle','earthCycle'],['Cetus Cycle','cetusCycle'],['Vallis Cycle','vallisCycle'],['Cambion Cycle','cambionCycle'],['Duviri Cycle','duviriCycle'],['Zariman Cycle','zarimanCycle']];
const stored = () => { try { return JSON.parse(localStorage.getItem('wfhub-settings') || '{}'); } catch { return {}; } };
const settings = stored();
let platform = settings.platform || 'pc', name = settings.name || 'Tenno', theme = settings.theme || 'system', page = 'Home', sequence = 0;

const esc = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const title = text => text.replace(/-/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
const safeUrl = value => { try { const u = new URL(value); return ['http:', 'https:'].includes(u.protocol) ? u.href : ''; } catch { return ''; } };
const get = async url => { const response = await fetch(url); if (!response.ok) throw Error(`HTTP ${response.status}`); return response.json(); };
const tryGet = url => get(url).catch(() => null);

function applyTheme() {
  const dark = theme === 'dark' || (theme === 'system' && !matchMedia('(prefers-color-scheme: light)').matches);
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
  document.querySelector('meta[name=theme-color]').content = dark ? '#000000' : '#ffffff';
}
matchMedia('(prefers-color-scheme: light)').addEventListener('change', applyTheme);
applyTheme();

function left(expiry) {
  const end = Date.parse(expiry); if (!end) return '';
  const s = Math.max(0, Math.floor((end - Date.now()) / 1000));
  return `${Math.floor(s / 86400)}d ${Math.floor(s % 86400 / 3600)}h ${Math.floor(s % 3600 / 60)}m`;
}
const countdown = expiry => expiry ? `<b data-expiry="${esc(expiry)}">${left(expiry)}</b>` : '';
setInterval(() => document.querySelectorAll('[data-expiry]').forEach(el => el.textContent = left(el.dataset.expiry)), 30000);

const spinner = '<div class="spinner" role="status" aria-label="Loading"></div>';
const fail = error => `<p class="empty">Unable to load data. ${esc(error.message)}<br>Please try again.</p>`;
const heading = (text, sub = '') => `<h1 class="title">${esc(text)}</h1>${sub ? `<p class="subtitle">${esc(sub)}</p>` : ''}`;
const topbar = backTo => backTo ? `<div class="topbar"><button class="back" data-back>‹ Back</button></div>` : '<div class="topbar"><a class="site" href="/">← Website</a></div>';
const section = (text, sub, action) => `<div class="section"><div><h2>${esc(text)}</h2><p>${esc(sub)}</p></div>${action ? `<button data-go="${action}">View all</button>` : ''}</div>`;
const searchBox = placeholder => `<label class="search">${svg('search')}<input type="search" placeholder="${esc(placeholder)}" aria-label="${esc(placeholder)}"></label>`;
const row = (label, desc = '', image = '', icon = '') => `<button class="row" type="button"><span class="ic">${image ? `<img src="${esc(image)}" alt="" loading="lazy" onerror="this.remove()">` : icon}</span><span class="tx"><strong>${esc(label)}</strong><small>${esc(desc)}</small></span><span class="chev">›</span></button>`;

function bind(selector, callback) {
  content.querySelectorAll(selector).forEach((el, i) => { el.onclick = () => callback(el, i); });
}
function show(html, backTo) {
  content.innerHTML = `${topbar(backTo)}<div class="screen">${html}</div>`;
  window.scrollTo(0, 0);
  const button = content.querySelector('[data-back]');
  if (button) button.onclick = backTo;
}
function renderTabs() {
  tabs.innerHTML = pages.map(([label, icon]) => `<button class="${page === label ? 'on' : ''}" data-page="${label}" aria-label="${label}">${svg(icon)}${label}</button>`).join('');
  tabs.querySelectorAll('[data-page]').forEach(b => b.onclick = () => openPage(b.dataset.page));
}
function openPage(next) {
  page = next; sequence++; renderTabs(); history.replaceState(null, '', `#${next.toLowerCase()}`);
  ({ Home: home, Activities: activityList, Market: market, Codex: codex, Settings: settingsPage })[next]();
}

async function home() {
  const id = sequence;
  show(`${heading(`Welcome, ${name}`)}${spinner}`);
  const base = `https://api.warframestat.us/${platform}/`;
  const [banners, nightwave, trader, events, alerts, news] = await Promise.all([tryGet(`${cdn}api/events.json`), ...['nightwave', 'voidTrader', 'events', 'alerts', 'news'].map(e => tryGet(`${base}${e}/`))]);
  if (id !== sequence) return;
  const parts = [heading(`Welcome, ${name}`)];
  for (const b of Array.isArray(banners) ? banners : []) {
    const link = safeUrl(String(b.link || '').replace('{lang}', 'en')); if (!b.title || !link) continue;
    parts.push(`<a class="banner" href="${esc(link)}" target="_blank" rel="noopener">${b.imageLink ? `<img src="${esc(b.imageLink)}" alt="">` : ''}<div><strong>${esc(b.title)}</strong></div></a>`);
  }
  if (nightwave && !Array.isArray(nightwave)) parts.push(`<button class="feature" data-activity="nightwave"><strong>NIGHTWAVE</strong><span>Nora’s Mix Season ${esc(nightwave.season ?? '–')}</span>${countdown(nightwave.expiry)}</button>`);
  if (Array.isArray(events) && events.length) parts.push(section('Events', 'Latest Events', 'events'), `<div class="stack">${events.map(e => {
    const progress = e.maximumScore > 0 ? Math.min(100, Math.max(0, 100 - e.currentScore / e.maximumScore * 100)) : (typeof e.health === 'number' ? e.health : null);
    return `<div class="card"><div class="top-line"><h3>${esc(e.description || 'Event')}</h3>${progress != null ? `<span class="pct">${progress.toFixed(2)}%</span>` : ''}</div><div class="meta"><span>${esc(e.node || e.victimNode || '–')}</span>${countdown(e.expiry)}</div></div>`; }).join('')}</div>`);
  if (Array.isArray(news) && news.length) {
    const sorted = [...news].sort((a, b) => String(b.date).localeCompare(String(a.date))).slice(0, 2);
    parts.push(section('News', 'Latest news', 'news'), `<div class="stack">${sorted.map((n, i) => {
      const text = n.translations?.en || n.message || 'News', link = safeUrl(n.link), date = n.date ? new Date(n.date).toLocaleString() : '';
      const inner = `<div class="img">${n.imageLink ? `<img src="${esc(n.imageLink)}" alt="" loading="lazy">` : ''}<span class="n">${i + 1}</span></div><h3>${esc(text)}</h3><small>${esc(date)}</small>`;
      return link ? `<a class="news" href="${esc(link)}" target="_blank" rel="noopener">${inner}</a>` : `<div class="news">${inner}</div>`; }).join('')}</div>`);
  }
  if (trader && !Array.isArray(trader)) parts.push(`<button class="banner" data-activity="voidTrader" style="width:100%;margin-top:18px"><img src="${cdn}sections/baro-ki-teer.png" alt="" onerror="this.remove()"><div><strong>${esc(trader.character || 'Baro Ki’Teer')}</strong><span>${esc(trader.location || '')}</span><br>${countdown(trader.expiry)}</div></button>`);
  if (Array.isArray(alerts) && alerts.length) parts.push(section('Activities', 'Latest Activities', 'alerts'), `<div class="stack">${alerts.map(a => {
    const m = a.mission || {}, r = m.reward || {}, items = r.items?.length ? r.items : (r.countedItems || []).map(c => `${c.count} ${c.type}`);
    return `<div class="card"><h3>${esc(a.tag === 'LotusGift' ? 'Gifts of the Lotus' : (m.node || 'Alert'))}</h3><div class="meta"><span>${esc(items.join(', '))}</span>${countdown(a.expiry)}</div></div>`; }).join('')}</div>`);
  parts.push(section('Codex', 'Codex content', 'Codex'), `<div class="art-grid">${[['Warframes', 'warframes', 'excalibur-prime'], ['Secondary', 'secondary', 'secondary']].map(([t, c, img]) => `<button class="art" data-category="${c}"><img src="${cdn}codex/${img}.png" alt="" loading="lazy"><strong>${t}</strong></button>`).join('')}</div>`);
  if (!nightwave && !events && !alerts && !news && !trader) parts.push('<p class="empty">Some content could not be loaded.</p>');
  show(parts.join(''));
  bind('[data-go]', b => { const t = b.dataset.go; if (t === 'Codex') openPage('Codex'); else activity(activities.find(a => a[1] === t)); });
  bind('[data-activity]', b => activity(activities.find(a => a[1] === b.dataset.activity)));
  bind('[data-category]', b => category(b.dataset.category, () => openPage('Home')));
}

function activityList() {
  show(`${heading('Activities', 'Worldstate details')}${searchBox('Search activities')}<div class="list" id="list"></div>`);
  const input = content.querySelector('input');
  const update = () => {
    const found = activities.filter(([n]) => n.toLowerCase().includes(input.value.toLowerCase()));
    content.querySelector('#list').innerHTML = found.map(([n, e]) => row(n, 'View activity', `${cdn}activities/${e}.png`, '✳')).join('') || '<p class="empty">No activities found.</p>';
    bind('#list .row', (_, i) => activity(found[i]));
  };
  input.oninput = update; update();
}

async function activity([label, endpoint]) {
  const id = ++sequence, backTo = () => openPage(page);
  show(`${heading(label, 'Live worldstate')}${spinner}`, backTo);
  try {
    const data = await get(`https://api.warframestat.us/${platform}/${endpoint}/`); if (id !== sequence) return;
    const items = Array.isArray(data) ? data : [data];
    const body = items.length ? `<div class="stack">${items.map(item => {
      const head = item.node || item.mission?.node || item.name || item.title || item.state || item.message || label;
      const fields = Object.entries(item).filter(([k, v]) => !['id', '_id', 'node', 'name', 'title', 'message'].includes(k) && ['string', 'number', 'boolean'].includes(typeof v)).slice(0, 6).map(([k, v]) => `${title(k)}: ${v}`).join(' · ');
      return `<div class="card"><h3>${esc(head)}</h3>${fields ? `<p>${esc(fields)}</p>` : ''}</div>`; }).join('')}</div>` : '<p class="empty">Nothing active right now.</p>';
    show(`${heading(label, 'Live worldstate')}${body}`, backTo);
  } catch (e) { if (id === sequence) show(`${heading(label, 'Live worldstate')}${fail(e)}`, backTo); }
}

function codex() {
  show(`${heading('Codex', 'Codex content')}<div class="art-grid wide">${categories.map(c => `<button class="art" data-category="${c}"><img src="${cdn}codex/${c}.png" alt="" loading="lazy"><strong>${esc(title(c))}</strong></button>`).join('')}</div>`);
  bind('[data-category]', b => category(b.dataset.category, () => openPage('Codex')));
}

async function category(cat, backTo) {
  const id = ++sequence;
  show(`${heading(title(cat), 'Browse items')}${searchBox(`Search ${title(cat)}`)}${spinner}`, backTo);
  try {
    const items = await get(`/api/${cat}.json`); if (id !== sequence) return;
    show(`${heading(title(cat), 'Browse items')}${searchBox(`Search ${title(cat)}`)}<div class="list" id="list"></div>`, backTo);
    const input = content.querySelector('input');
    const update = () => {
      const found = items.filter(x => x.name.toLowerCase().includes(input.value.toLowerCase()));
      content.querySelector('#list').innerHTML = found.map(x => row(x.name, 'View details', x.imageName, '✳')).join('') || '<p class="empty">No items found.</p>';
      bind('#list .row', (_, i) => detail(cat, found[i].slug, backTo));
    };
    input.oninput = update; update();
  } catch (e) { if (id === sequence) show(`${heading(title(cat), 'Browse items')}${fail(e)}`, backTo); }
}

async function detail(cat, slug, parentBack) {
  const id = ++sequence, backTo = () => category(cat, parentBack);
  show(`${heading(title(slug), 'Codex')}${spinner}`, backTo);
  try {
    const item = await get(`/api/${cat}/${slug}.json`); if (id !== sequence) return;
    const stats = ['health', 'shield', 'armor', 'power', 'masteryReq', 'sprintSpeed'].filter(k => item[k] != null);
    show(`${heading(item.name, 'Codex')}<div class="card">${item.imageName ? `<img class="detail-img" src="${esc(item.imageName)}" alt="">` : ''}<p style="margin:0">${esc(item.description || 'No description available.')}</p>${stats.length ? `<div class="stats">${stats.map(k => `<div class="stat">${esc(title(k))}<b>${esc(item[k])}</b></div>`).join('')}</div>` : ''}</div>${item.abilities?.length ? `${section('Abilities', '')}<div class="list">${item.abilities.map(a => row(a.name, a.description)).join('')}</div>` : ''}`, backTo);
  } catch (e) { if (id === sequence) show(`${heading('Codex')}${fail(e)}`, backTo); }
}

async function market() {
  const id = sequence;
  const head = `${heading('Market')}<div class="seg" role="tablist"><button class="on" data-kind="sell">Sell</button><button data-kind="buy">Buy</button></div>`;
  show(`${head}<div id="orders">${spinner}</div>`);
  try {
    const [orders, items] = await Promise.all([get('https://api.warframe.market/v2/orders/recent'), get('https://api.warframe.market/v2/items')]);
    if (id !== sequence) return;
    const lookup = new Map(items.data.map(x => [x.id, x]));
    const render = kind => {
      content.querySelectorAll('[data-kind]').forEach(b => b.classList.toggle('on', b.dataset.kind === kind));
      const entries = orders.data.filter(x => x.type === kind).slice(0, 35);
      content.querySelector('#orders').innerHTML = entries.length ? `<div class="stack">${entries.map(o => {
        const item = lookup.get(o.itemId), info = item?.i18n?.en, slug = item?.slug;
        const inner = `<span class="thumb">${info?.icon ? `<img src="https://warframe.market/static/assets/${esc(info.icon)}" alt="" loading="lazy">` : ''}</span><span class="info"><strong>${esc(info?.name || 'Item')}</strong><small>${esc(o.user?.ingameName || '–')}</small><small>×${esc(o.quantity ?? 0)}</small></span><span class="price"><b>◆ ${esc(o.platinum ?? 0)}</b><small>PLATINUM</small></span>`;
        return slug ? `<a class="order" href="https://warframe.market/items/${esc(slug)}" target="_blank" rel="noopener">${inner}</a>` : `<div class="order">${inner}</div>`; }).join('')}</div>` : '<p class="empty">No recent orders.</p>';
    };
    bind('[data-kind]', b => render(b.dataset.kind)); render('sell');
  } catch (e) { if (id === sequence) content.querySelector('#orders').innerHTML = fail(e); }
}

function settingsPage() {
  const option = (value, label, current) => `<option value="${value}" ${current === value ? 'selected' : ''}>${label}</option>`;
  show(`${heading('Settings')}<div class="form-label">Profile</div><form id="form"><div class="list"><div class="field"><div><label for="name">Tenno Name</label><input id="name" maxlength="60" value="${esc(name)}"></div></div><div class="field"><div><label for="platform">Platform</label><select id="platform">${[['pc', 'PC'], ['ps4', 'PlayStation'], ['xb1', 'Xbox'], ['swi', 'Nintendo Switch']].map(([v, n]) => option(v, n, platform)).join('')}</select></div></div></div><div class="form-label">Preferences</div><div class="list"><div class="field"><div><label for="theme">Appearance</label><select id="theme">${option('system', 'System', theme)}${option('light', 'Light', theme)}${option('dark', 'Dark', theme)}</select></div></div></div><button class="primary" type="submit">Save</button><p class="saved" id="saved" role="status"></p></form>`);
  content.querySelector('#theme').onchange = e => { theme = e.target.value; applyTheme(); };
  content.querySelector('#form').onsubmit = e => {
    e.preventDefault();
    name = content.querySelector('#name').value.trim() || 'Tenno';
    platform = content.querySelector('#platform').value;
    theme = content.querySelector('#theme').value; applyTheme();
    localStorage.setItem('wfhub-settings', JSON.stringify({ name, platform, theme }));
    content.querySelector('#saved').textContent = 'Settings saved.';
  };
}

const start = pages.map(([p]) => p).find(p => p.toLowerCase() === location.hash.slice(1)) || 'Home';
openPage(start);

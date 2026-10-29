import { cp, mkdir, rm, writeFile } from 'node:fs/promises';

const output = new URL('../.cloudflare-build/', import.meta.url);
const web = new URL('../apps/wfhub-web/', import.meta.url);
const categories = ['Arcanes', 'Arch-Gun', 'Arch-Melee', 'Archwing', 'Enemy', 'Fish', 'Gear', 'Glyphs', 'Melee', 'Misc', 'Mods', 'Node', 'Pets', 'Primary', 'Quests', 'Railjack', 'Relics', 'Resources', 'Secondary', 'Sentinels', 'SentinelWeapons', 'Sigils', 'Skins', 'Warframes'];
const source = 'https://raw.githubusercontent.com/WFCD/warframe-items/master/data/';
const fields = ['uniqueName', 'name', 'description', 'health', 'shield', 'armor', 'stamina', 'power', 'masteryReq', 'sprintSpeed', 'abilities', 'sprint', 'wikiaUrl', 'releaseDate'];

await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
await cp(web, output, { recursive: true, filter: path => path !== new URL('api/', web).pathname.replace(/\/$/, '') });
await cp(new URL('api/swagger/', web), new URL('api/swagger/', output), { recursive: true });
await mkdir(new URL('_api-data/', output), { recursive: true });
for (const category of categories) {
  const response = await fetch(`${source}json/${category}.json`);
  if (!response.ok) throw new Error(`Failed to download ${category}: HTTP ${response.status}`);
  const items = await response.json();
  if (!Array.isArray(items)) throw new Error(`Invalid ${category} data`);
  const list = [];
  const details = {};
  for (const item of items) {
    if (typeof item.name !== 'string') continue;
    const slug = item.name.normalize('NFKD').replace(/[\u0300-\u036f]/g, '').toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    if (!slug || Object.hasOwn(details, slug)) continue;
    const imageName = item.imageName ? `${source}img/${encodeURIComponent(item.imageName)}` : '';
    list.push({ name: item.name, imageName, slug });
    details[slug] = Object.fromEntries(fields.map(field => [field, item[field] ?? (field === 'abilities' ? [] : null)]));
    Object.assign(details[slug], { imageName, slug });
  }
  await writeFile(new URL(`_api-data/${category.toLowerCase()}.json`, output), JSON.stringify({ list, details }));
  console.log(`${category}: ${list.length} items`);
}

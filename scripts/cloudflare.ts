import { cp, mkdir, readdir, writeFile } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const web = join(root, 'apps/wfhub-web');
const output = join(root, 'dist/client');
const assetData = join(output, '_api-data');
const source = 'https://raw.githubusercontent.com/WFCD/warframe-items/master/data/';
const categories = [
  'Arcanes', 'Arch-Gun', 'Arch-Melee', 'Archwing', 'Enemy', 'Fish', 'Gear', 'Glyphs',
  'Melee', 'Misc', 'Mods', 'Node', 'Pets', 'Primary', 'Quests', 'Railjack', 'Relics',
  'Resources', 'Secondary', 'Sentinels', 'SentinelWeapons', 'Sigils', 'Skins', 'Warframes',
];
const fields = [
  'uniqueName', 'name', 'description', 'health', 'shield', 'armor', 'stamina', 'power',
  'masteryReq', 'sprintSpeed', 'abilities', 'sprint', 'wikiaUrl', 'releaseDate',
];

type Item = Record<string, unknown> & { name?: unknown; imageName?: unknown };

/** Preserve the existing Codex slugs and discard duplicate names. */
function makeCatalogue(items: Item[]) {
  const list: { name: string; imageName: string; slug: string }[] = [];
  const details: Record<string, Record<string, unknown>> = Object.create(null);
  for (const item of items) {
    if (typeof item.name !== 'string') continue;
    const slug = item.name.normalize('NFKD').replace(/[\u0300-\u036f]/g, '')
      .toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
    if (!slug || Object.hasOwn(details, slug)) continue;
    const imageName = typeof item.imageName === 'string' && item.imageName
      ? `${source}img/${encodeURIComponent(item.imageName)}` : '';
    list.push({ name: item.name, imageName, slug });
    details[slug] = Object.fromEntries(fields.map(field => [field, item[field] ?? (field === 'abilities' ? [] : null)]));
    Object.assign(details[slug], { imageName, slug });
  }
  return { list, details };
}

async function prepareAssets() {
  const result = spawnSync('bunx', ['vite', 'build', '--config', 'vite.config.ts'], { cwd: root, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Vite build failed (${result.status ?? 'interrupted'})`);
  await mkdir(assetData, { recursive: true });
  await mkdir(join(output, 'api/swagger'), { recursive: true });
  const dataDir = join(web, 'api/data');
  const snapshots = (await readdir(dataDir)).filter(name => name === 'events.json' || /^mastery-[a-z-]+\.json$/.test(name));
  await Promise.all([
    cp(join(web, 'api/swagger/openapi.json'), join(output, 'api/swagger/openapi.json')),
    ...snapshots.map(name => cp(join(dataDir, name), join(assetData, name))),
  ]);
  const swagger = await Bun.build({ entrypoints: [join(web, 'api/swagger/init.ts')], outdir: join(output, 'api/swagger'), target: 'browser' });
  if (!swagger.success) throw new Error(`Failed to build Swagger: ${swagger.logs.join('\n')}`);
}

async function buildCategory(category: string) {
  const response = await fetch(`${source}json/${category}.json`);
  if (!response.ok) throw new Error(`Failed to download ${category}: HTTP ${response.status}`);
  const items: unknown = await response.json();
  if (!Array.isArray(items)) throw new Error(`Invalid ${category} data`);
  const catalogue = makeCatalogue(items);
  await writeFile(join(assetData, `${category.toLowerCase()}.json`), JSON.stringify(catalogue));
  console.log(`${category}: ${catalogue.list.length} items`);
}

async function build() {
  await prepareAssets();
  // Limit concurrent downloads to avoid overwhelming the upstream server.
  let next = 0;
  await Promise.all(Array.from({ length: 6 }, async () => {
    while (next < categories.length) await buildCategory(categories[next++]);
  }));
}

// Apply migrations against the same configuration and environment as the deploy.
function migrationOptions(args: string[]): string[] {
  const options: string[] = [];
  const withValue = new Set(['--env', '-e', '--config', '-c', '--cwd', '--env-file', '--profile']);
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (withValue.has(arg)) {
      if (!args[i + 1] || args[i + 1].startsWith('-')) throw new Error(`Missing value for ${arg}`);
      options.push(arg, args[++i]);
    } else if ([...withValue].some(option => arg.startsWith(`${option}=`))) {
      options.push(arg);
    }
  }
  return options;
}

function runWrangler(args: string[]): void {
  const result = spawnSync('bunx', ['wrangler', ...args], { cwd: root, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Wrangler failed (${result.status ?? 'interrupted'}): ${args[0]}`);
}

async function main() {
  const [action, ...args] = process.argv.slice(2);
  if (action !== 'build' && action !== 'deploy') {
    console.error('Usage: bun scripts/cloudflare.ts {build|deploy} [wrangler options]');
    process.exitCode = 2;
    return;
  }
  await build();
  if (action === 'deploy') {
    if (!args.includes('--dry-run')) {
      runWrangler(['d1', 'migrations', 'apply', 'DB', '--remote', ...migrationOptions(args)]);
    }
    runWrangler(['deploy', ...args]);
  }
}

main().catch(error => { console.error(error); process.exitCode = 1; });

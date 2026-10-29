import { cp, mkdir, rm } from 'node:fs/promises';

const output = new URL('../.cloudflare-build/', import.meta.url);
const web = new URL('../apps/wfhub-web/', import.meta.url);
const api = new URL('../apps/wfhub-api/api/', import.meta.url);

await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
await cp(web, output, { recursive: true });
try {
  await cp(api, new URL('api/', output), { recursive: true });
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
  console.warn('API data directory is missing; /api/ requests will return 404.');
}

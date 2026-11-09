import { readFile, stat } from 'node:fs/promises';
import { dirname, extname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import worker from '../apps/wfhub-web/api/worker.ts';
import { transformWithOxc } from 'vite';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const web = join(root, 'apps/wfhub-web');
const build = join(root, 'dist/client');
const types: Record<string, string> = {
  '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json',
  '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg', '.webp': 'image/webp', '.ico': 'image/x-icon',
};

/** Resolve only paths inside the requested asset directory. */
function fileFor(base: string, pathname: string): string | null {
  const path = resolve(base, `.${decodeURIComponent(pathname)}`);
  return path.startsWith(base + sep) ? path : null;
}

async function fileResponse(path: string | null): Promise<Response> {
  if (!path) return new Response('Not found', { status: 404 });
  try {
    const data = await readFile(path);
    return new Response(data, { headers: { 'Content-Type': types[extname(path)] || 'application/octet-stream' } });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT' || (error as NodeJS.ErrnoException).code === 'EISDIR') {
      return new Response('Not found', { status: 404 });
    }
    throw error;
  }
}

const assets = {
  async fetch(request: Request) {
    const path = new URL(request.url).pathname;
    if (path === '/api/swagger/openapi.json') return fileResponse(join(web, 'api/swagger/openapi.json'));
    if (path === '/_api-data/events.json' || /^\/_api-data\/mastery-[a-z-]+\.json$/.test(path)) {
      return fileResponse(fileFor(join(web, 'api/data'), path.replace('/_api-data/', '/')));
    }
    return fileResponse(fileFor(build, path));
  },
};

export async function handle(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname;
  if (path === '/theme.js' || path === '/api/swagger/init.js') {
    const source = await readFile(join(web, path === '/theme.js' ? 'theme.ts' : 'api/swagger/init.ts'), 'utf8');
    const result = await transformWithOxc(source, path === '/theme.js' ? 'theme.ts' : 'init.ts');
    return new Response(result.code, {
      headers: { 'Content-Type': 'text/javascript' },
    });
  }
  if (path === '/api') return Response.redirect(`${url.origin}/api/${url.search}`, 308);
  if (path === '/api/swagger') return Response.redirect(`${url.origin}/api/swagger/`, 308);
  if (path.startsWith('/api/')) {
    const response = await worker.fetch(request, { ASSETS: assets });
    if (response.status !== 404 || path.startsWith('/api/mastery/') || path === '/api/events.json') return response;
    const category = path.split('/')[2]?.replace(/\.json$/, '');
    const categoryFile = category && fileFor(build, `/_api-data/${category}.json`);
    if (categoryFile) {
      try { await stat(categoryFile); return response; }
      catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
    }
    try { return await fetch(`https://wfhub.cagatayldzz.com${path}${url.search}`, { method: request.method }); }
    catch { return new Response('API data unavailable. Run npm run cf:build first.', { status: 503 }); }
  }
  if (path.startsWith('/api') || path.startsWith('/_api-data')) return new Response('Not found', { status: 404 });
  return fileResponse(fileFor(web, path.endsWith('/') ? `${path}index.html` : path));
}


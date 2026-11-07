import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { dirname, extname, join, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import worker from '../apps/wfhub-web/api/worker.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const web = join(root, 'apps/wfhub-web');
const build = join(root, '.cloudflare-build');
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
    if (path === '/_api-data/events.json' || /^\/_api-data\/mastery-[a-z-]+\.json$/.test(path)) {
      return fileResponse(fileFor(join(web, 'api/data'), path.replace('/_api-data/', '/')));
    }
    return fileResponse(fileFor(build, path));
  },
};

async function handle(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname;
  if (path === '/api') return Response.redirect(`${url.origin}/api/${url.search}`, 308);
  if (path === '/api/') return fileResponse(join(web, 'api/swagger/index.html'));
  if (path === '/api/swagger') return Response.redirect(`${url.origin}/api/swagger/`, 308);
  if (['/api/swagger/', '/api/swagger/index.html', '/api/swagger/openapi.json'].includes(path)) {
    return fileResponse(fileFor(web, path.endsWith('/') ? `${path}index.html` : path));
  }
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

function startWebServer() {
  const port = Number(process.env.PORT || 8000);
  createServer(async (req, res) => {
    try {
      const response = await handle(new Request(`http://localhost:${port}${req.url}`, { method: req.method, headers: req.headers as HeadersInit }));
      res.writeHead(response.status, Object.fromEntries(response.headers));
      res.end(req.method === 'HEAD' ? undefined : Buffer.from(await response.arrayBuffer()));
    } catch (error) {
      console.error(error);
      res.writeHead(500).end('Internal server error');
    }
  }).listen(port, () => {
    const url = `http://localhost:${port}/`;
    console.log(`WFHub running at ${url}`);
    if (process.platform === 'darwin' && !process.env.CI) spawn('open', [url], { stdio: 'ignore', detached: true }).unref();
  });
}

const [action, ...args] = process.argv.slice(2);
if (action === 'web') startWebServer();
else if (action === 'phone') {
  const child = spawn('cargo', ['run', '-p', 'wfhub', '--', '--phone', ...args], { cwd: root, stdio: 'inherit' });
  child.on('error', error => { console.error(error); process.exitCode = 1; });
  child.on('exit', (code, signal) => { if (signal) process.kill(process.pid, signal); else process.exitCode = code ?? 1; });
} else {
  console.error('Usage: node scripts/dev.ts {phone|web} [options]');
  process.exitCode = 2;
}

#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
node --input-type=module - "$@" <<'NODE'
import { resolve, sep } from 'node:path';
import worker from './apps/wfhub-web/api/worker.mjs';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { spawn } from 'node:child_process';

const root = resolve('apps/wfhub-web');
const build = resolve('.cloudflare-build');
function fileFor(base, pathname) {
  const path = resolve(base, `.${decodeURIComponent(pathname)}`);
  return path.startsWith(base + sep) ? path : null;
}
async function handle(request) {
    const url = new URL(request.url);
    if (url.pathname === '/api/swagger') return Response.redirect(`${url.origin}/api/swagger/`, 308);
    if (['/api/swagger/', '/api/swagger/index.html', '/api/swagger/openapi.json'].includes(url.pathname)) {
      const path = url.pathname.endsWith('/') ? `${url.pathname}index.html` : url.pathname;
      const file = fileFor(root, path);
      return file && await exists(file) ? await fileResponse(file) : new Response('Not found', { status: 404 });
    }
    if (url.pathname.startsWith('/api/')) {
      if (url.pathname === '/api/events.json') {
        return worker.fetch(request, { ASSETS: { async fetch() {
          return await fileResponse(resolve(root, 'api/data/events.json'));
        } } });
      }
      const category = url.pathname.split('/')[2]?.replace(/\.json$/, '');
      const data = fileFor(build, `/_api-data/${category}.json`);
      if (data && await exists(data)) {
        return worker.fetch(request, { ASSETS: { async fetch(assetRequest) {
          const file = fileFor(build, new URL(assetRequest.url).pathname);
          return file && await exists(file) ? await fileResponse(file) : new Response('Not found', { status: 404 });
        } } });
      }
      try { return await fetch(`https://wfhub.cagatayldzz.com${url.pathname}${url.search}`); }
      catch { return new Response('API data unavailable. Run bash scripts/cf-build.sh first.', { status: 503 }); }
    }
    if (url.pathname.startsWith('/api') || url.pathname.startsWith('/_api-data')) return new Response('Not found', { status: 404 });
    let path = url.pathname;
    if (path.endsWith('/')) path += 'index.html';
    const file = fileFor(root, path);
    return file && await exists(file) ? await fileResponse(file) : new Response('Not found', { status: 404 });
}
async function exists(path) { try { await readFile(path); return true; } catch { return false; } }
async function fileResponse(path) {
  const ext = path.split('.').pop();
  const types = { html: 'text/html', css: 'text/css', js: 'text/javascript', json: 'application/json', svg: 'image/svg+xml', png: 'image/png', jpg: 'image/jpeg', webp: 'image/webp', ico: 'image/x-icon' };
  return new Response(await readFile(path), { headers: { 'Content-Type': types[ext] || 'application/octet-stream' } });
}
const port = Number(process.env.PORT || 8000);
createServer(async (req, res) => {
  try {
    const response = await handle(new Request(`http://localhost:${port}${req.url}`, { method: req.method, headers: req.headers }));
    res.writeHead(response.status, Object.fromEntries(response.headers));
    res.end(Buffer.from(await response.arrayBuffer()));
  } catch (error) { console.error(error); res.writeHead(500); res.end('Internal server error'); }
}).listen(port, () => {
  const url = `http://localhost:${port}/`;
  console.log(`WFHub running at ${url}`);
  if (process.platform === 'darwin') spawn('open', [url], { stdio: 'ignore', detached: true }).unref();
});
NODE

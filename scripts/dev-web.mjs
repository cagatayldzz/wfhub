import { resolve, sep } from 'node:path';
import worker from '../apps/wfhub-web/api/worker.mjs';

const root = resolve('apps/wfhub-web');
const build = resolve('.cloudflare-build');
function fileFor(base, pathname) {
  const path = resolve(base, `.${decodeURIComponent(pathname)}`);
  return path.startsWith(base + sep) ? Bun.file(path) : null;
}
const server = Bun.serve({
  port: Number(process.env.PORT || 8000),
  async fetch(request) {
    const url = new URL(request.url);
    if (url.pathname === '/api/swagger') return Response.redirect(`${url.origin}/api/swagger/`, 308);
    if (['/api/swagger/', '/api/swagger/index.html', '/api/swagger/openapi.json'].includes(url.pathname)) {
      const path = url.pathname.endsWith('/') ? `${url.pathname}index.html` : url.pathname;
      const file = fileFor(root, path);
      return file && await file.exists() ? new Response(file) : new Response('Not found', { status: 404 });
    }
    if (url.pathname.startsWith('/api/')) {
      const category = url.pathname.split('/')[2]?.replace(/\.json$/, '');
      const data = fileFor(build, `/_api-data/${category}.json`);
      if (data && await data.exists()) {
        return worker.fetch(request, { ASSETS: { async fetch(assetRequest) {
          const file = fileFor(build, new URL(assetRequest.url).pathname);
          return file && await file.exists() ? new Response(file) : new Response('Not found', { status: 404 });
        } } });
      }
      try { return await fetch(`https://wfhub.cagatayldzz.com${url.pathname}${url.search}`); }
      catch { return new Response('API data unavailable. Run bun run cf:build first.', { status: 503 }); }
    }
    if (url.pathname.startsWith('/api') || url.pathname.startsWith('/_api-data')) return new Response('Not found', { status: 404 });
    let path = url.pathname;
    if (path === '/hub') return Response.redirect(`${url.origin}/hub/`, 302);
    if (path.endsWith('/')) path += 'index.html';
    const file = fileFor(root, path);
    return file && await file.exists() ? new Response(file) : new Response('Not found', { status: 404 });
  },
});
console.log(`WFHub running at ${server.url}`);
if (process.platform === 'darwin') Bun.spawn(['open', server.url.href], { stdout: 'ignore', stderr: 'ignore' });

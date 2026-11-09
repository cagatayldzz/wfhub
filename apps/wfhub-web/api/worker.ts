import { BASE_HEADERS, JSON_HEADERS } from './http';
import { submitFeedback, type WorkerEnv } from './feedback';

const SWAGGER_PATHS = new Set(['/api/swagger/', '/api/swagger/index.html', '/api/swagger/openapi.json', '/api/swagger/init.js']);
const MASTERY_CATEGORIES = new Set([
  'warframes', 'primary', 'secondary', 'melee', 'kitgun', 'zaw', 'amp', 'sentinel',
  'moa', 'hound', 'beast', 'robotic', 'archwing', 'archgun', 'archmelee',
  'necramech', 'railjack', 'k-drive',
  'railjack-intrinsic', 'drifter-intrinsic', 'starchart', 'steel-path', 'founders', 'china-founders', 'updated',
]);

function notFound() {
  return new Response('Not found\n', { status: 404, headers: BASE_HEADERS });
}

/** Fetches a bundled asset using GET, including for incoming HEAD requests. */
async function fetchAsset(request: Request, env: WorkerEnv, path: string) {
  const url = new URL(request.url);
  url.pathname = path;
  return env.ASSETS.fetch(new Request(url, { method: 'GET' }));
}

/** Omits the body for HEAD while retaining the JSON response headers. */
function jsonResponse(request: Request, body: BodyInit | null) {
  return new Response(request.method === 'HEAD' ? null : body, { headers: JSON_HEADERS });
}

async function serveJsonAsset(request: Request, env: WorkerEnv, path: string) {
  const response = await fetchAsset(request, env, path);
  return response.ok ? jsonResponse(request, response.body) : notFound();
}

/** Resolves a category list or item from its bundled catalogue index. */
async function serveCatalogue(request: Request, env: WorkerEnv, path: string) {
  const match = /^\/api\/([a-z0-9-]+)(?:\/([a-z0-9-]+))?\.json$/.exec(path);
  if (!match) return notFound();

  const [, category, slug] = match;
  const response = await fetchAsset(request, env, `/_api-data/${category}.json`);
  if (!response.ok) return notFound();

  const data = await response.json() as { details: Record<string, unknown>; list: unknown };
  const item = slug ? data.details[slug] : data.list;
  return item ? jsonResponse(request, JSON.stringify(item)) : notFound();
}

export default {
  async fetch(request: Request, env: WorkerEnv) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (path === '/api/feedback' && request.method === 'POST') {
      return submitFeedback(request, env);
    }

    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response('Method not allowed\n', { status: 405, headers: BASE_HEADERS });
    }

    if (path === '/api') {
      url.pathname = '/api/';
      return Response.redirect(url, 308);
    }
    if (path === '/api/') {
      url.pathname = '/api/swagger/';
      return env.ASSETS.fetch(new Request(url, request));
    }
    if (path === '/api/swagger') {
      url.pathname += '/';
      return Response.redirect(url, 308);
    }
    if (SWAGGER_PATHS.has(path)) {
      if (path.endsWith('/index.html')) url.pathname = '/api/swagger/';
      return env.ASSETS.fetch(new Request(url, request));
    }

    if (path === '/api/events.json') {
      return serveJsonAsset(request, env, '/_api-data/events.json');
    }

    const mastery = /^\/api\/mastery\/([a-z-]+)\.json$/.exec(path);
    if (mastery && MASTERY_CATEGORIES.has(mastery[1])) {
      return serveJsonAsset(request, env, `/_api-data/mastery-${mastery[1]}.json`);
    }

    return serveCatalogue(request, env, path);
  },
};

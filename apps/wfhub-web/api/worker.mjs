const BASE_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'X-Content-Type-Options': 'nosniff',
};

const JSON_HEADERS = { ...BASE_HEADERS, 'Content-Type': 'application/json; charset=utf-8' };
const SWAGGER_PATHS = new Set(['/api/swagger/', '/api/swagger/index.html', '/api/swagger/openapi.json']);
const MASTERY_CATEGORIES = new Set([
  'warframes', 'primary', 'secondary', 'melee', 'kitgun', 'zaw', 'amp', 'sentinel',
  'moa', 'hound', 'beast', 'robotic', 'archwing', 'archgun', 'archmelee',
  'necramech', 'railjack', 'k-drive',
]);

function notFound() {
  return new Response('Not found\n', { status: 404, headers: BASE_HEADERS });
}

async function fetchAsset(request, env, path) {
  const url = new URL(request.url);
  url.pathname = path;
  return env.ASSETS.fetch(new Request(url, { method: 'GET' }));
}

function jsonResponse(request, body) {
  return new Response(request.method === 'HEAD' ? null : body, { headers: JSON_HEADERS });
}

async function serveJsonAsset(request, env, path) {
  const response = await fetchAsset(request, env, path);
  return response.ok ? jsonResponse(request, response.body) : notFound();
}

async function serveCatalogue(request, env, path) {
  const match = /^\/api\/([a-z0-9-]+)(?:\/([a-z0-9-]+))?\.json$/.exec(path);
  if (!match) return notFound();

  const [, category, slug] = match;
  const response = await fetchAsset(request, env, `/_api-data/${category}.json`);
  if (!response.ok) return notFound();

  const data = await response.json();
  const item = slug ? data.details[slug] : data.list;
  return item ? jsonResponse(request, JSON.stringify(item)) : notFound();
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response('Method not allowed\n', { status: 405, headers: BASE_HEADERS });
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

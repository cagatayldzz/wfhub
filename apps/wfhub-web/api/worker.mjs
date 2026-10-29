export default {
  async fetch(request, env) {
    const path = new URL(request.url).pathname;
    const headers = {
      'Access-Control-Allow-Origin': '*',
      'X-Content-Type-Options': 'nosniff',
    };

    if (request.method !== 'GET' && request.method !== 'HEAD') {
      return new Response('Method not allowed\n', { status: 405, headers });
    }
    const suffix = path.slice('/api/'.length);
    if (!path.startsWith('/api/') || !suffix.endsWith('.json') ||
        !suffix.split('/').every(part => part !== '.' && part !== '..' && /^[a-zA-Z0-9._-]+$/.test(part))) {
      return new Response('Not found\n', { status: 404, headers });
    }
    const match = /^([a-z0-9-]+)(?:\/([a-z0-9-]+))?\.json$/.exec(suffix);
    if (!match) return new Response('Not found\n', { status: 404, headers });
    const url = new URL(request.url);
    url.pathname = `/_api-data/${match[1]}.json`;
    const response = await env.ASSETS.fetch(new Request(url, { method: 'GET' }));
    if (!response.ok) return new Response('Not found\n', { status: 404, headers });
    const data = await response.json();
    const result = match[2] ? data.details[match[2]] : data.list;
    if (!result) return new Response('Not found\n', { status: 404, headers });
    headers['Content-Type'] = 'application/json; charset=utf-8';
    return new Response(request.method === 'HEAD' ? null : JSON.stringify(result), { headers });
  },
};

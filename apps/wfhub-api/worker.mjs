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
    const response = await env.ASSETS.fetch(request);
    const responseHeaders = new Headers(response.headers);
    for (const [name, value] of Object.entries(headers)) responseHeaders.set(name, value);
    return new Response(response.body, { status: response.status, statusText: response.statusText, headers: responseHeaders });
  },
};

import { describe, expect, test } from 'bun:test';
import worker from './worker.ts';
import type { WorkerEnv } from './feedback.ts';

const catalogue = { list: [{ name: 'Excalibur', slug: 'excalibur' }], details: { excalibur: { name: 'Excalibur' } } };
const assets: Record<string, unknown> = {
  '/_api-data/events.json': [{ title: 'Event' }],
  '/_api-data/mastery-warframes.json': [{ name: 'Excalibur' }],
  '/_api-data/warframes.json': catalogue,
};
const env: WorkerEnv = {
  ASSETS: {
    async fetch(request) {
      const asset = assets[new URL(request.url).pathname];
      return asset === undefined ? new Response('Not found', { status: 404 }) : Response.json(asset);
    },
  },
};

function request(path: string, method = 'GET') {
  return worker.fetch(new Request(`https://wfhub.example${path}`, { method }), env);
}

describe('API routes', () => {
  test('serves event and mastery snapshots with CORS headers', async () => {
    const events = await request('/api/events.json?test=1');
    expect(events.status).toBe(200);
    expect(events.headers.get('access-control-allow-origin')).toBe('*');
    expect(await events.json()).toEqual(assets['/_api-data/events.json']);
    const mastery = await request('/api/mastery/warframes.json');
    expect(await mastery.json()).toEqual(assets['/_api-data/mastery-warframes.json']);
  });

  test('serves category lists and individual items', async () => {
    expect(await (await request('/api/warframes.json')).json()).toEqual(catalogue.list);
    expect(await (await request('/api/warframes/excalibur.json')).json()).toEqual(catalogue.details.excalibur);
    expect((await request('/api/warframes/unknown.json')).status).toBe(404);
  });

  test('HEAD returns the same status and content type without a body', async () => {
    for (const path of ['/api/events.json', '/api/mastery/warframes.json', '/api/warframes/excalibur.json']) {
      const get = await request(path);
      const head = await request(path, 'HEAD');
      expect(head.status).toBe(get.status);
      expect(head.headers.get('content-type')).toBe(get.headers.get('content-type'));
      expect(await head.text()).toBe('');
    }
  });

  test('rejects invalid paths and unsupported methods', async () => {
    for (const path of ['/api/unknown.json', '/api/warframes/extra/path.json', '/api/warframes.txt']) {
      expect((await request(path)).status).toBe(404);
    }
    expect((await request('/api/events.json', 'POST')).status).toBe(405);
    expect((await request('/api/feedback', 'POST')).status).toBe(415);
  });

  test('serves API documentation', async () => {
    const response = await request('/api/swagger/');
    expect(response.status).toBe(200);
    expect(await response.text()).toContain('swagger-ui');
  });

  test('accepts valid feedback and enforces the rate limit', async () => {
    let allowed = true;
    const writes: string[] = [];
    const feedbackEnv: WorkerEnv = {
      ...env,
      FEEDBACK_RATE_LIMIT_SECRET: 'test-secret',
      DB: {
        prepare(query) {
          return {
            bind() { return this; },
            async first() { return allowed ? { count: 1 } : null; },
            async run() { writes.push(query); return {}; },
          };
        },
      },
    };
    const send = () => worker.fetch(new Request('https://wfhub.example/api/feedback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'CF-Connecting-IP': '192.0.2.1' },
      body: JSON.stringify({ title: 'Example', category: 'bug', message: 'A reproducible issue' }),
    }), feedbackEnv);
    expect((await send()).status).toBe(201);
    expect(writes.some(query => query.includes('INSERT INTO feedback '))).toBe(true);
    allowed = false;
    expect((await send()).status).toBe(429);
  });
});

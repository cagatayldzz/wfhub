import { JSON_HEADERS } from './http.mjs';

const MAX_BODY_BYTES = 8192;
const RATE_WINDOW_SECONDS = 60 * 60;
const RATE_RETENTION_SECONDS = 48 * 60 * 60;
const CATEGORIES = new Set(['bug', 'suggestion', 'other']);
const FIELDS = new Set(['title', 'category', 'message']);
const CONTROL_CHARACTERS = /[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/;

class RequestError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

function response(status, error) {
  const body = error ? { error } : { ok: true };
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...JSON_HEADERS, 'Cache-Control': 'no-store' },
  });
}

function validateHeaders(request) {
  if (!/^application\/json(?:\s*;|$)/i.test(request.headers.get('Content-Type') || '')) {
    throw new RequestError(415, 'Unsupported content type');
  }
  if (Number(request.headers.get('Content-Length') || 0) > MAX_BODY_BYTES) {
    throw new RequestError(413, 'Request too large');
  }

}

async function readJson(request) {
  const reader = request.body?.getReader();
  if (!reader) throw new RequestError(400, 'Invalid request');

  const chunks = [];
  let size = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    size += value.length;
    if (size > MAX_BODY_BYTES) {
      await reader.cancel();
      throw new RequestError(413, 'Request too large');
    }
    chunks.push(value);
  }

  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }

  try {
    return JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch {
    throw new RequestError(400, 'Invalid JSON');
  }
}

function validateFeedback(data) {
  if (!data || typeof data !== 'object' || Array.isArray(data) ||
      Object.keys(data).some(key => !FIELDS.has(key)) ||
      typeof data.title !== 'string' || typeof data.message !== 'string' ||
      !CATEGORIES.has(data.category)) {
    throw new RequestError(400, 'Invalid feedback');
  }

  const title = data.title.trim();
  const message = data.message.trim();
  const titleLength = [...title].length;
  const messageLength = [...message].length;
  if (titleLength < 3 || titleLength > 120 || messageLength < 10 || messageLength > 4000 ||
      CONTROL_CHARACTERS.test(title + message)) {
    throw new RequestError(400, 'Invalid feedback');
  }
  return { title, message, category: data.category };
}

async function clientRateKey(secret, ip) {
  const encoder = new TextEncoder();
  const day = new Date().toISOString().slice(0, 10);
  const key = await crypto.subtle.importKey(
    'raw', encoder.encode(secret), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign'],
  );
  const digest = await crypto.subtle.sign('HMAC', key, encoder.encode(`${day}:${ip}`));
  return Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, '0')).join('');
}

async function consumeRateLimit(db, clientKey, now) {
  const windowStart = now - RATE_WINDOW_SECONDS;
  const result = await db.prepare(`
    INSERT INTO feedback_rate_limits (client_key, window_start, count)
    VALUES (?, ?, 1)
    ON CONFLICT(client_key) DO UPDATE SET
      count = CASE WHEN window_start <= ? THEN 1 ELSE count + 1 END,
      window_start = CASE WHEN window_start <= ? THEN excluded.window_start ELSE window_start END
    WHERE window_start <= ? OR count < 5
    RETURNING count
  `).bind(clientKey, now, windowStart, windowStart, windowStart).first();
  return result !== null;
}

async function saveFeedback(db, feedback) {
  await db.prepare(`
    INSERT INTO feedback (id, category, title, message)
    VALUES (?, ?, ?, ?)
  `).bind(crypto.randomUUID(), feedback.category, feedback.title, feedback.message).run();
}

async function cleanRateLimits(db, now) {
  try {
    await db.prepare('DELETE FROM feedback_rate_limits WHERE window_start < ?')
      .bind(now - RATE_RETENTION_SECONDS).run();
  } catch {
    // A cleanup failure must not report a successfully saved submission as failed.
  }
}

export async function submitFeedback(request, env) {
  try {
    validateHeaders(request);
    const ip = request.headers.get('CF-Connecting-IP');
    if (!ip) throw new RequestError(403, 'Request unavailable');

    const feedback = validateFeedback(await readJson(request));
    if (!env.FEEDBACK_RATE_LIMIT_SECRET) throw new RequestError(503, 'Service unavailable');

    // Rotate the keyed IP hash daily so the database does not contain raw IP addresses.
    const clientKey = await clientRateKey(env.FEEDBACK_RATE_LIMIT_SECRET, ip);
    const now = Math.floor(Date.now() / 1000);
    if (!await consumeRateLimit(env.DB, clientKey, now)) {
      throw new RequestError(429, 'Too many requests');
    }

    await saveFeedback(env.DB, feedback);
    await cleanRateLimits(env.DB, now);
    return response(201);
  } catch (error) {
    if (error instanceof RequestError) return response(error.status, error.message);
    return response(503, 'Service unavailable');
  }
}

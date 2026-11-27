// GET of an https address through an HTTP proxy (CONNECT tunnel), from a Worker. `fetch` cannot use a proxy and workers-rs's
// `start_tls()` cannot say which host the certificate must match, so the tunnel is done here, with the sockets API.
// The game's server answers 403 to Cloudflare's addresses; the proxy's addresses it accepts.
import { connect } from "cloudflare:sockets";

const MAX_BYTES = 3_000_000;
const TIMEOUT_MS = 20_000;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

function indexOfBytes(bytes, text, from = 0) {
  const needle = encoder.encode(text);
  outer: for (let i = from; i <= bytes.length - needle.length; i++) {
    for (let j = 0; j < needle.length; j++) if (bytes[i + j] !== needle[j]) continue outer;
    return i;
  }
  return -1;
}

function concat(parts) {
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let at = 0;
  for (const part of parts) {
    out.set(part, at);
    at += part.length;
  }
  return out;
}

/** Reads until `done(bytesSoFar)` says it is enough or the stream ends; returns everything read. */
async function readWhile(reader, done) {
  const parts = [];
  let size = 0;
  for (;;) {
    const { value, done: ended } = await reader.read();
    if (value) {
      parts.push(value);
      size += value.length;
      if (size > MAX_BYTES) throw new Error("answer too large");
    }
    const all = concat(parts);
    if (ended || done(all)) return all;
  }
}

function unchunk(body) {
  const parts = [];
  let at = 0;
  for (;;) {
    const lineEnd = indexOfBytes(body, "\r\n", at);
    if (lineEnd < 0) throw new Error("bad chunked body");
    const size = parseInt(decoder.decode(body.slice(at, lineEnd)).split(";")[0], 16);
    if (Number.isNaN(size)) throw new Error("bad chunk size");
    if (size === 0) return concat(parts);
    parts.push(body.slice(lineEnd + 2, lineEnd + 2 + size));
    at = lineEnd + 2 + size + 2;
  }
}

async function run(proxyUrl, host, path) {
  const proxy = new URL(proxyUrl);
  const socket = connect({ hostname: proxy.hostname, port: Number(proxy.port || 80) }, { secureTransport: "starttls", allowHalfOpen: false });
  let writer = socket.writable.getWriter();
  let reader = socket.readable.getReader();

  const credentials = btoa(`${decodeURIComponent(proxy.username)}:${decodeURIComponent(proxy.password)}`);
  await writer.write(
    encoder.encode(`CONNECT ${host}:443 HTTP/1.1\r\nHost: ${host}:443\r\nProxy-Authorization: Basic ${credentials}\r\nProxy-Connection: keep-alive\r\n\r\n`),
  );
  const reply = decoder.decode(await readWhile(reader, (bytes) => indexOfBytes(bytes, "\r\n\r\n") >= 0));
  const statusLine = reply.split("\r\n")[0];
  if (!/^HTTP\/1\.[01] 200/.test(statusLine)) throw new Error(`proxy CONNECT: ${statusLine.slice(0, 80) || "no answer"}`);

  writer.releaseLock();
  reader.releaseLock();
  const tls = socket.startTls({ expectedServerHostname: host });
  writer = tls.writable.getWriter();
  reader = tls.readable.getReader();
  await writer.write(
    encoder.encode(`GET ${path} HTTP/1.1\r\nHost: ${host}\r\nAccept-Encoding: gzip\r\nAccept: application/json\r\nConnection: close\r\n\r\n`),
  );
  const raw = await readWhile(reader, () => false);
  socket.close().catch(() => {});

  const split = indexOfBytes(raw, "\r\n\r\n");
  if (split < 0) throw new Error(`game answer: no headers (${raw.length} bytes: ${JSON.stringify(decoder.decode(raw.slice(0, 120)))})`);
  const head = decoder.decode(raw.slice(0, split)).split("\r\n");
  const status = Number(head[0].split(" ")[1]);
  const headers = Object.fromEntries(head.slice(1).map((line) => [line.slice(0, line.indexOf(":")).toLowerCase(), line.slice(line.indexOf(":") + 1).trim()]));
  let body = raw.slice(split + 4);
  if (status !== 200) throw new Error(`game answered ${status}`);
  if ((headers["transfer-encoding"] || "").includes("chunked")) body = unchunk(body);
  if (headers["content-encoding"] === "gzip") {
    body = new Uint8Array(await new Response(new Blob([body]).stream().pipeThrough(new DecompressionStream("gzip"))).arrayBuffer());
  }
  return decoder.decode(body);
}

/** The body of `https://{host}{path}`, fetched through the proxy at `proxyUrl` (http://user:password@host:port). */
export async function fetch_via_proxy(proxyUrl, host, path) {
  let timer;
  const timeout = new Promise((_, reject) => {
    timer = setTimeout(() => reject(new Error("timed out")), TIMEOUT_MS);
  });
  try {
    return await Promise.race([run(proxyUrl, host, path), timeout]);
  } finally {
    clearTimeout(timer);
  }
}

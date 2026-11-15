//! An HTTPS `GET` through an HTTP proxy: a `CONNECT` tunnel over the Worker's TCP sockets (`cloudflare:sockets`), TLS to the
//! target inside it, then the request. `fetch()` cannot use a proxy, and workers-rs's `Socket::start_tls` cannot name the
//! host the certificate must match (the target's, not the proxy's), so this part is a small piece of JavaScript, kept here.
//! Everything around it is Rust.

use base64::{Engine, engine::general_purpose::STANDARD};
use worker::Url;
use worker::js_sys::Promise;
use worker::wasm_bindgen::prelude::*;
use worker::wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(
  wasm_bindgen = worker::wasm_bindgen,
  inline_js = r#"
import { connect } from "cloudflare:sockets";

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function find(bytes, pattern, from = 0) {
  outer: for (let i = from; i <= bytes.length - pattern.length; i++) {
    for (let j = 0; j < pattern.length; j++) if (bytes[i + j] !== pattern[j]) continue outer;
    return i;
  }
  return -1;
}

function join(a, b) {
  const out = new Uint8Array(a.length + b.length);
  out.set(a, 0);
  out.set(b, a.length);
  return out;
}

const HEADER_END = [13, 10, 13, 10];

async function readUntilHeaderEnd(reader, limit) {
  let bytes = new Uint8Array(0);
  for (;;) {
    const end = find(bytes, HEADER_END);
    if (end >= 0) return { head: decoder.decode(bytes.slice(0, end)), rest: bytes.slice(end + 4) };
    if (bytes.length > limit) throw new Error("answer headers too long");
    const { value, done } = await reader.read();
    if (done) throw new Error("connection closed before the answer headers");
    bytes = join(bytes, value);
  }
}

async function readAll(reader, initial, limit) {
  let bytes = initial;
  for (;;) {
    const { value, done } = await reader.read();
    if (done) return bytes;
    bytes = join(bytes, value);
    if (bytes.length > limit) throw new Error("answer too large");
  }
}

function dechunk(bytes) {
  const parts = [];
  let at = 0;
  for (;;) {
    const lineEnd = find(bytes, [13, 10], at);
    if (lineEnd < 0) throw new Error("bad chunked body");
    const size = parseInt(decoder.decode(bytes.slice(at, lineEnd)).split(";")[0].trim(), 16);
    if (Number.isNaN(size)) throw new Error("bad chunk size");
    if (size === 0) break;
    parts.push(bytes.slice(lineEnd + 2, lineEnd + 2 + size));
    at = lineEnd + 2 + size + 2;
  }
  return parts.reduce((all, part) => join(all, part), new Uint8Array(0));
}

async function gunzip(bytes) {
  const stream = new DecompressionStream("gzip");
  const writer = stream.writable.getWriter();
  writer.write(bytes);
  writer.close();
  return new Uint8Array(await new Response(stream.readable).arrayBuffer());
}

async function run(proxyHost, proxyPort, auth, host, path) {
  const socket = connect({ hostname: proxyHost, port: proxyPort }, { secureTransport: "starttls", allowHalfOpen: false });
  try {
    let writer = socket.writable.getWriter();
    let reader = socket.readable.getReader();
    await writer.write(encoder.encode(
      `CONNECT ${host}:443 HTTP/1.1\r\nHost: ${host}:443\r\nProxy-Authorization: Basic ${auth}\r\n\r\n`));
    const connectAnswer = await readUntilHeaderEnd(reader, 8192);
    const status = connectAnswer.head.split("\r\n")[0];
    if (!/^HTTP\/1\.[01] 200/.test(status)) throw new Error(`the proxy answered: ${status}`);
    writer.releaseLock();
    reader.releaseLock();

    const tls = socket.startTls({ expectedServerHostname: host });
    writer = tls.writable.getWriter();
    reader = tls.readable.getReader();
    await writer.write(encoder.encode(
      `GET ${path} HTTP/1.1\r\nHost: ${host}\r\nAccept: application/json\r\nAccept-Encoding: gzip\r\nUser-Agent: WFHub\r\nConnection: close\r\n\r\n`));
    const { head, rest } = await readUntilHeaderEnd(reader, 16384);
    const [statusLine, ...lines] = head.split("\r\n");
    const code = parseInt(statusLine.split(" ")[1], 10);
    if (code !== 200) throw new Error(`the game answered ${code}`);
    const headers = {};
    for (const line of lines) {
      const i = line.indexOf(":");
      if (i > 0) headers[line.slice(0, i).trim().toLowerCase()] = line.slice(i + 1).trim().toLowerCase();
    }
    let body = await readAll(reader, rest, 4_000_000);
    if (headers["transfer-encoding"]?.includes("chunked")) body = dechunk(body);
    if (headers["content-encoding"]?.includes("gzip")) body = await gunzip(body);
    return decoder.decode(body);
  } finally {
    try { socket.close(); } catch (_) {}
  }
}

export function proxied_get(proxyHost, proxyPort, auth, host, path) {
  const timeout = new Promise((_, reject) => setTimeout(() => reject(new Error("timed out")), 25000));
  return Promise.race([run(proxyHost, proxyPort, auth, host, path), timeout]);
}
"#
)]
extern "C" {
  fn proxied_get(proxy_host: &str, proxy_port: u16, auth: &str, host: &str, path: &str) -> Promise;
}

/// `GET https://{host}{path}` through the proxy `http://user:password@host:port`: the body of a `200` answer.
pub async fn get(proxy: &str, host: &str, path: &str) -> Result<String, String> {
  let proxy = Url::parse(proxy).map_err(|_| "WORLDSTATE_PROXY is not an address (http://user:password@host:port)".to_string())?;
  let proxy_host = proxy.host_str().ok_or("WORLDSTATE_PROXY has no host")?;
  let proxy_port = proxy.port_or_known_default().ok_or("WORLDSTATE_PROXY has no port")?;
  let auth = STANDARD.encode(format!("{}:{}", proxy.username(), proxy.password().unwrap_or("")));
  let answer = JsFuture::from(proxied_get(proxy_host, proxy_port, &auth, host, path)).await.map_err(|error| {
    let text = error.dyn_ref::<worker::js_sys::Error>().map(|e| String::from(e.message())).or_else(|| error.as_string());
    text.unwrap_or_else(|| "the request through the proxy failed".to_string())
  })?;
  answer.as_string().ok_or_else(|| "the answer is not text".to_string())
}

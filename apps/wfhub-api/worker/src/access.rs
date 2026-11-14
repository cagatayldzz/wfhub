//! Cloudflare Access check for the admin routes. Access already stops anonymous requests at the edge; this verifies the
//! `Cf-Access-Jwt-Assertion` token again (signature, issuer, audience, expiry) so a mistake in the Access policy cannot
//! open the data. Anything missing or wrong means "not authorized".

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::Value;
use worker::js_sys::{self, Function, Reflect, Uint8Array};
use worker::wasm_bindgen::{JsCast, JsValue};
use worker::wasm_bindgen_futures::JsFuture;
use worker::*;

fn decode(part: &str) -> Option<Vec<u8>> {
  URL_SAFE_NO_PAD.decode(part).ok()
}

fn call(target: &JsValue, method: &str, args: &[JsValue]) -> Option<JsValue> {
  let function: Function = Reflect::get(target, &method.into()).ok()?.into();
  let array = js_sys::Array::new();
  for arg in args {
    array.push(arg);
  }
  function.apply(target, &array).ok()
}

async fn resolve(value: JsValue) -> Option<JsValue> {
  let promise: js_sys::Promise<JsValue> = value.dyn_into().ok()?;
  JsFuture::from(promise).await.ok()
}

/// `true` only for a valid Access token issued for this application.
pub async fn is_authorized(req: &Request, env: &Env) -> bool {
  check(req, env).await.is_some()
}

async fn check(req: &Request, env: &Env) -> Option<()> {
  let team = env.var("ACCESS_TEAM_DOMAIN").ok()?.to_string();
  let audience = env.var("ACCESS_AUD").ok()?.to_string();
  if !team.ends_with(".cloudflareaccess.com") || team.contains(['/', ':', '@', ' ']) || audience.is_empty() {
    return None;
  }
  let token = req.headers().get("cf-access-jwt-assertion").ok()??;
  let parts: Vec<&str> = token.split('.').collect();
  let [header, payload, signature] = parts.as_slice() else { return None };

  let header: Value = serde_json::from_slice(&decode(header)?).ok()?;
  if header["alg"] != "RS256" {
    return None;
  }
  let kid = header["kid"].as_str()?;
  let claims: Value = serde_json::from_slice(&decode(payload)?).ok()?;
  let now = (Date::now().as_millis() / 1000) as i64;
  let audience_ok = match &claims["aud"] {
    Value::String(value) => *value == audience,
    Value::Array(values) => values.iter().any(|value| value.as_str() == Some(audience.as_str())),
    _ => false,
  };
  if claims["iss"].as_str()? != format!("https://{team}") || !audience_ok || claims["exp"].as_i64()? <= now {
    return None;
  }
  if claims["nbf"].as_i64().is_some_and(|nbf| nbf > now + 60) {
    return None;
  }

  let mut certs = Fetch::Url(Url::parse(&format!("https://{team}/cdn-cgi/access/certs")).ok()?).send().await.ok()?;
  if certs.status_code() != 200 {
    return None;
  }
  let certs: Value = certs.json().await.ok()?;
  let jwk = certs["keys"].as_array()?.iter().find(|key| key["kid"].as_str() == Some(kid) && key["kty"] == "RSA")?;

  let subtle = Reflect::get(&Reflect::get(&js_sys::global(), &"crypto".into()).ok()?, &"subtle".into()).ok()?;
  let algorithm = js_sys::JSON::parse(r#"{"name":"RSASSA-PKCS1-v1_5","hash":"SHA-256"}"#).ok()?;
  let usages = js_sys::Array::of1(&"verify".into());
  let key = resolve(call(&subtle, "importKey", &["jwk".into(), js_sys::JSON::parse(&jwk.to_string()).ok()?, algorithm.clone(), false.into(), usages.into()])?).await?;
  let signature = Uint8Array::from(decode(signature)?.as_slice());
  let signed = Uint8Array::from(format!("{}.{}", parts[0], parts[1]).as_bytes());
  let valid = resolve(call(&subtle, "verify", &[algorithm, key, signature.into(), signed.into()])?).await?;
  valid.as_bool()?.then_some(())
}

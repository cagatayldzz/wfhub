use wfhub_api::feedback::{check_headers, client_key, parse};

fn body(title: &str, category: &str, message: &str) -> Vec<u8> {
  serde_json::json!({ "title": title, "category": category, "message": message }).to_string().into_bytes()
}

#[test]
fn accepts_good_feedback_and_trims_it() {
  let feedback = parse(&body("  Crash  ", "bug", "  It crashes when I open the codex.  ")).unwrap();
  assert_eq!((feedback.title.as_str(), feedback.category.as_str()), ("Crash", "bug"));
  assert_eq!(feedback.message, "It crashes when I open the codex.");
}

#[test]
fn refuses_bad_feedback() {
  let long = "x".repeat(121);
  for bad in [
    body("ab", "bug", "long enough message"),
    body(&long, "bug", "long enough message"),
    body("Title", "bug", "short"),
    body("Title", "nonsense", "long enough message"),
    body("Title", "bug", "bad \u{1} control character"),
    br#"{"title":"Title","category":"bug","message":"long enough message","extra":1}"#.to_vec(),
    br#"["not","an","object"]"#.to_vec(),
    br#"{"title":1,"category":"bug","message":"long enough message"}"#.to_vec(),
  ] {
    assert_eq!(parse(&bad).unwrap_err(), (400, "Invalid feedback"), "{}", String::from_utf8_lossy(&bad));
  }
  assert_eq!(parse(b"{").unwrap_err(), (400, "Invalid JSON"));
  assert_eq!(parse(&vec![b' '; 9000]).unwrap_err().0, 413);
}

#[test]
fn counts_letters_not_bytes() {
  assert!(parse(&body("Çağ", "other", "ğüşiöçĞÜŞİÖÇ!")).is_ok());
}

#[test]
fn checks_headers() {
  assert!(check_headers(Some("application/json"), Some("100")).is_ok());
  assert!(check_headers(Some("Application/JSON; charset=utf-8"), None).is_ok());
  assert_eq!(check_headers(Some("text/plain"), None).unwrap_err().0, 415);
  assert_eq!(check_headers(Some("application/jsonx"), None).unwrap_err().0, 415);
  assert_eq!(check_headers(None, None).unwrap_err().0, 415);
  assert_eq!(check_headers(Some("application/json"), Some("9000")).unwrap_err().0, 413);
}

#[test]
fn client_key_changes_with_the_day_and_the_client() {
  let key = client_key("secret", "2026-11-10", "203.0.113.5");
  assert_eq!(key.len(), 64);
  assert_eq!(key, client_key("secret", "2026-11-10", "203.0.113.5"));
  assert_ne!(key, client_key("secret", "2026-11-11", "203.0.113.5"));
  assert_ne!(key, client_key("secret", "2026-11-10", "203.0.113.6"));
  assert_ne!(key, client_key("other", "2026-11-10", "203.0.113.5"));
}

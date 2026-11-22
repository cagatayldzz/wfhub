use crate::feedback_url;

#[derive(serde::Serialize)]
struct Submission<'a> {
  title: &'a str,
  category: &'a str,
  message: &'a str,
}

/// The form can be sent: a title of 3-120 and a message of 10-4000 characters (not counting the outer spaces).
#[uniffi::export]
pub fn feedback_is_valid(title: String, message: String) -> bool {
  (3..=120).contains(&title.trim().chars().count()) && (10..=4000).contains(&message.trim().chars().count())
}

/// Address and JSON body of the POST request. `category` is `bug`, `suggestion` or `other`.
#[derive(Clone, Debug, uniffi::Record)]
pub struct FeedbackRequest {
  pub url: String,
  pub body: String,
}

#[uniffi::export]
pub fn feedback_request(title: String, category: String, message: String) -> FeedbackRequest {
  let category = if ["bug", "suggestion", "other"].contains(&category.as_str()) { category.as_str() } else { "other" };
  let body = serde_json::to_string(&Submission { title: &title, category, message: &message }).unwrap_or_default();
  FeedbackRequest { url: feedback_url(), body }
}

/// `None` when the server accepted the feedback (HTTP 201), otherwise the translation key of the error to show.
#[uniffi::export]
pub fn feedback_error_key(status: u16) -> Option<String> {
  match status {
    201 => None,
    429 => Some("feedbackTooManyRequests".into()),
    _ => Some("feedbackSendFailed".into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn validates_and_builds() {
    assert!(feedback_is_valid("Bug".into(), "It crashes on start".into()));
    assert!(!feedback_is_valid("  a ".into(), "It crashes on start".into()));
    assert!(!feedback_is_valid("Bug".into(), "short".into()));
    let request = feedback_request("T\"x".into(), "nope".into(), "m".into());
    assert!(request.body.contains(r#""category":"other""#) && request.body.contains(r#"T\"x"#));
    assert_eq!(feedback_error_key(201), None);
    assert_eq!(feedback_error_key(429).as_deref(), Some("feedbackTooManyRequests"));
  }
}

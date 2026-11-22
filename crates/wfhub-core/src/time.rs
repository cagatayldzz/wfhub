use chrono::DateTime;

/// Unix seconds of an ISO 8601 date such as `2025-01-02T03:04:05.000Z` (fraction optional).
#[uniffi::export]
pub fn expiry_unix(expiry: String) -> Option<i64> {
  DateTime::parse_from_rfc3339(&expiry).ok().map(|date| date.timestamp())
}

/// "2d 3h 4m" until `expiry`, "0d 0h 0m" once it passed, `None` when the date is not readable.
#[uniffi::export]
pub fn countdown(expiry: String, now_unix: i64) -> Option<String> {
  let seconds = (expiry_unix(expiry)? - now_unix).max(0);
  Some(format!("{}d {}h {}m", seconds / 86_400, seconds % 86_400 / 3_600, seconds % 3_600 / 60))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn counts_down() {
    let end = "2025-01-02T00:00:00.000Z".to_string();
    let end_unix = expiry_unix(end.clone()).unwrap();
    assert_eq!(countdown(end.clone(), end_unix - 90_060).as_deref(), Some("1d 1h 1m"));
    assert_eq!(countdown(end, end_unix + 5).as_deref(), Some("0d 0h 0m"));
    assert_eq!(countdown("nope".into(), 0), None);
    assert!(expiry_unix("2025-01-02T00:00:00Z".into()).is_some());
  }
}

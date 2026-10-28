use std::ffi::{CString, c_char};

/// Returns an owned, newline-separated route catalogue. Release it with `wfhub_free`.
#[unsafe(no_mangle)]
pub extern "C" fn wfhub_catalogue() -> *mut c_char {
  let mut lines = Vec::new();
  for (name, endpoint) in wfhub_core::ACTIVITIES {
    lines.push(format!("activity\t{name}\t{endpoint}"));
  }
  for category in wfhub_core::CATEGORIES {
    lines.push(format!("category\t{category}"));
  }
  CString::new(lines.join("\n")).expect("catalogue contains no NUL").into_raw()
}

/// Frees a pointer returned by `wfhub_catalogue`.
///
/// # Safety
/// The pointer must be null or a live pointer returned by `wfhub_catalogue`, freed once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wfhub_free(pointer: *mut c_char) {
  if !pointer.is_null() {
    // SAFETY: The caller guarantees the pointer came from CString::into_raw and is freed once.
    drop(unsafe { CString::from_raw(pointer) });
  }
}

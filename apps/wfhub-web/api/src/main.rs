use std::{
  borrow::Cow,
  env, fs,
  io::{self, BufRead, BufReader, Read, Write},
  net::{TcpListener, TcpStream},
  path::{Path, PathBuf},
  time::Duration,
};

fn resolve_path(root: &Path, target: &str) -> Option<PathBuf> {
  let path = target.split('?').next()?;
  let suffix = path.strip_prefix("/api/")?;
  if suffix.is_empty()
    || !suffix.ends_with(".json")
    || !suffix.split('/').all(|part| {
      !part.is_empty() && part != "." && part != ".." && part.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
    })
  {
    return None;
  }
  Some(root.join(suffix))
}

fn mastery_data(path: &str) -> Option<&'static [u8]> {
  match path {
    "/api/mastery/warframes.json" => Some(include_bytes!("../data/mastery-warframes.json")),
    "/api/mastery/primary.json" => Some(include_bytes!("../data/mastery-primary.json")),
    "/api/mastery/secondary.json" => Some(include_bytes!("../data/mastery-secondary.json")),
    "/api/mastery/melee.json" => Some(include_bytes!("../data/mastery-melee.json")),
    "/api/mastery/kitgun.json" => Some(include_bytes!("../data/mastery-kitgun.json")),
    "/api/mastery/zaw.json" => Some(include_bytes!("../data/mastery-zaw.json")),
    "/api/mastery/amp.json" => Some(include_bytes!("../data/mastery-amp.json")),
    "/api/mastery/sentinel.json" => Some(include_bytes!("../data/mastery-sentinel.json")),
    "/api/mastery/moa.json" => Some(include_bytes!("../data/mastery-moa.json")),
    "/api/mastery/hound.json" => Some(include_bytes!("../data/mastery-hound.json")),
    "/api/mastery/beast.json" => Some(include_bytes!("../data/mastery-beast.json")),
    "/api/mastery/robotic.json" => Some(include_bytes!("../data/mastery-robotic.json")),
    "/api/mastery/archwing.json" => Some(include_bytes!("../data/mastery-archwing.json")),
    "/api/mastery/archgun.json" => Some(include_bytes!("../data/mastery-archgun.json")),
    "/api/mastery/archmelee.json" => Some(include_bytes!("../data/mastery-archmelee.json")),
    "/api/mastery/necramech.json" => Some(include_bytes!("../data/mastery-necramech.json")),
    "/api/mastery/railjack.json" => Some(include_bytes!("../data/mastery-railjack.json")),
    "/api/mastery/k-drive.json" => Some(include_bytes!("../data/mastery-k-drive.json")),
    "/api/mastery/railjack-intrinsic.json" => Some(include_bytes!("../data/mastery-railjack-intrinsic.json")),
    "/api/mastery/drifter-intrinsic.json" => Some(include_bytes!("../data/mastery-drifter-intrinsic.json")),
    "/api/mastery/starchart.json" => Some(include_bytes!("../data/mastery-starchart.json")),
    "/api/mastery/steel-path.json" => Some(include_bytes!("../data/mastery-steel-path.json")),
    "/api/mastery/founders.json" => Some(include_bytes!("../data/mastery-founders.json")),
    "/api/mastery/china-founders.json" => Some(include_bytes!("../data/mastery-china-founders.json")),
    "/api/mastery/updated.json" => Some(include_bytes!("../data/mastery-updated.json")),
    _ => None,
  }
}

struct Response {
  status: &'static str,
  content_type: &'static str,
  body: Cow<'static, [u8]>,
}

impl Response {
  fn json(body: impl Into<Cow<'static, [u8]>>) -> Self {
    Self { status: "200 OK", content_type: "application/json; charset=utf-8", body: body.into() }
  }

  fn error(status: &'static str, body: &'static [u8]) -> Self {
    Self { status, content_type: "text/plain; charset=utf-8", body: Cow::Borrowed(body) }
  }

  fn not_found() -> Self {
    Self::error("404 Not Found", b"Not found\n")
  }

  fn write_to(&self, output: &mut impl Write, head: bool) -> io::Result<()> {
    write!(
      output,
      "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
      self.status,
      self.content_type,
      self.body.len()
    )?;
    if !head {
      output.write_all(&self.body)?;
    }
    Ok(())
  }
}

fn route(method: &str, target: &str, root: &Path) -> Response {
  if method != "GET" && method != "HEAD" {
    return Response::error("405 Method Not Allowed", b"Method not allowed\n");
  }

  let path = target.split('?').next().unwrap_or("");
  if path == "/api/events.json" {
    return Response::json(&include_bytes!("../data/events.json")[..]);
  }
  if let Some(data) = mastery_data(path) {
    return Response::json(data);
  }
  let Some(path) = resolve_path(root, path) else { return Response::not_found() };
  match fs::read(path) {
    Ok(data) => Response::json(data),
    Err(error) if error.kind() == io::ErrorKind::NotFound => Response::not_found(),
    Err(_) => Response::error("500 Internal Server Error", b"Unable to read data\n"),
  }
}

fn serve(mut stream: TcpStream, root: &Path) -> io::Result<()> {
  stream.set_read_timeout(Some(Duration::from_secs(5)))?;
  let mut request = String::new();
  BufReader::new(&stream).take(8192).read_line(&mut request)?;
  let mut parts = request.split_whitespace();
  let method = parts.next().unwrap_or("");
  let target = parts.next().unwrap_or("");
  route(method, target, root).write_to(&mut stream, method == "HEAD")
}

fn main() -> io::Result<()> {
  let address = env::var("WFHUB_API_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
  let root = env::var_os("WFHUB_API_DATA_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("api"));
  let listener = TcpListener::bind(&address)?;
  eprintln!("WFHub API listening on http://{address}, serving {}", root.display());
  for stream in listener.incoming() {
    match stream {
      Ok(stream) => {
        let root = root.clone();
        std::thread::spawn(move || {
          if let Err(error) = serve(stream, &root) {
            eprintln!("Request failed: {error}");
          }
        });
      }
      Err(error) => eprintln!("Connection failed: {error}"),
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_accepts_api_json_paths() {
    let root = Path::new("data");
    assert_eq!(resolve_path(root, "/api/warframes.json?x=1"), Some(root.join("warframes.json")));
    assert_eq!(resolve_path(root, "/api/warframes/excalibur.json"), Some(root.join("warframes/excalibur.json")));
    for path in ["/api/../secret.json", "/api/%2e%2e/secret.json", "/api//x.json", "/api/x.txt", "/other/x.json"] {
      assert_eq!(resolve_path(root, path), None);
    }
  }

  #[test]
  fn serves_embedded_data_and_preserves_head_length() {
    let root = Path::new("missing-dataset");
    let get = route("GET", "/api/mastery/warframes.json?test=1", root);
    let head = route("HEAD", "/api/mastery/warframes.json", root);
    assert_eq!(get.status, "200 OK");
    assert!(!get.body.is_empty());
    let mut response = Vec::new();
    head.write_to(&mut response, true).unwrap();
    let headers = String::from_utf8(response).unwrap();
    assert!(headers.ends_with("\r\n\r\n"));
    assert!(headers.contains(&format!("Content-Length: {}\r\n", get.body.len())));
    assert_eq!(route("GET", "/api/events.json", root).status, "200 OK");
    assert_eq!(route("GET", "/api/unknown.json", root).status, "404 Not Found");
    assert_eq!(route("POST", "/api/events.json", root).status, "405 Method Not Allowed");
  }
}

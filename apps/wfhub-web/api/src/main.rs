use std::{
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

fn mastery_data(target: &str) -> Option<&'static [u8]> {
  match target.split('?').next()? {
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
    _ => None,
  }
}

fn serve(mut stream: TcpStream, root: &Path) -> io::Result<()> {
  stream.set_read_timeout(Some(Duration::from_secs(5)))?;
  let mut request = String::new();
  BufReader::new(&stream).take(8192).read_line(&mut request)?;
  let mut parts = request.split_whitespace();
  let method = parts.next().unwrap_or("");
  let target = parts.next().unwrap_or("");
  let (status, content_type, body) = if method != "GET" && method != "HEAD" {
    ("405 Method Not Allowed", "text/plain; charset=utf-8", b"Method not allowed\n".to_vec())
  } else if target.split('?').next() == Some("/api/events.json") {
    ("200 OK", "application/json; charset=utf-8", include_bytes!("../data/events.json").to_vec())
  } else if let Some(data) = mastery_data(target) {
    ("200 OK", "application/json; charset=utf-8", data.to_vec())
  } else if let Some(path) = resolve_path(root, target) {
    match fs::read(path) {
      Ok(data) => ("200 OK", "application/json; charset=utf-8", data),
      Err(error) if error.kind() == io::ErrorKind::NotFound => ("404 Not Found", "text/plain; charset=utf-8", b"Not found\n".to_vec()),
      Err(_) => ("500 Internal Server Error", "text/plain; charset=utf-8", b"Unable to read data\n".to_vec()),
    }
  } else {
    ("404 Not Found", "text/plain; charset=utf-8", b"Not found\n".to_vec())
  };
  write!(
    stream,
    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
    body.len()
  )?;
  if method != "HEAD" {
    stream.write_all(&body)?;
  }
  Ok(())
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
}

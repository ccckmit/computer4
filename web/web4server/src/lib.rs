use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;

pub fn run(addr: &str, root: &Path) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    let actual = listener.local_addr()?;
    let root = root.canonicalize()?;
    eprintln!("web4server listening on {actual}, root={}", root.display());
    run_on_listener(listener, &root)
}

pub fn run_on_listener(listener: TcpListener, root: &Path) -> std::io::Result<()> {
    let root = root.canonicalize()?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root = root.clone();
                thread::spawn(move || handle_client(stream, &root));
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, root: &Path) {
    let mut buf = [0; 4096];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request = String::from_utf8_lossy(&buf[..n]);

    let mut lines = request.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        let _ = respond(&mut stream, 400, b"Bad Request");
        return;
    }
    let method = parts[0];
    let raw_path = parts[1];

    if method != "GET" {
        let _ = respond(&mut stream, 405, b"Method Not Allowed");
        return;
    }

    // strip query string and fragment
    let path = raw_path.split('?').next().unwrap_or(raw_path);
    let path = path.split('#').next().unwrap_or(path);

    let file_path = resolve_path(root, path);
    let resolved = match file_path.and_then(|p| p.canonicalize().ok()) {
        Some(p) => p,
        None => {
            let _ = respond(&mut stream, 404, b"Not Found");
            return;
        }
    };

    // ensure it's within root (security)
    if !resolved.starts_with(root) {
        let _ = respond(&mut stream, 403, b"Forbidden");
        return;
    }

    // if directory, try index.html
    if resolved.is_dir() {
        let index = resolved.join("index.html");
        if index.exists() && index.is_file() {
            serve_file(&mut stream, &index);
        } else {
            let _ = respond(&mut stream, 403, b"Forbidden");
        }
        return;
    }

    serve_file(&mut stream, &resolved);
}

fn resolve_path(root: &Path, raw: &str) -> Option<PathBuf> {
    let clean = raw.trim_start_matches('/');
    if clean.is_empty() {
        return Some(root.to_path_buf());
    }
    let mut p = PathBuf::from(root);
    for component in Path::new(clean).components() {
        match component {
            std::path::Component::Normal(c) => p.push(c),
            _ => return None, // reject ".." and absolute components
        }
    }
    Some(p)
}

fn serve_file(stream: &mut TcpStream, path: &Path) {
    match fs::read(path) {
        Ok(body) => {
            let mime = content_type(path);
            let _ = respond_with_mime(stream, 200, mime, &body);
        }
        Err(_) => {
            let _ = respond(stream, 404, b"Not Found");
        }
    }
}

pub fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg" | "svgz") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("webp") => "image/webp",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("wasm") => "application/wasm",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        Some("gz") => "application/gzip",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    }
}

fn respond(stream: &mut TcpStream, status: u16, body: &[u8]) -> std::io::Result<()> {
    respond_with_mime(stream, status, "text/plain; charset=utf-8", body)
}

fn respond_with_mime(stream: &mut TcpStream, status: u16, mime: &str, body: &[u8]) -> std::io::Result<()> {
    let status_line = match status {
        200 => "200 OK",
        400 => "400 Bad Request",
        403 => "403 Forbidden",
        404 => "404 Not Found",
        405 => "405 Method Not Allowed",
        413 => "413 Content Too Large",
        _ => "500 Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nContent-Type: {mime}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut resp = Vec::with_capacity(header.len() + body.len());
    resp.extend_from_slice(header.as_bytes());
    resp.extend_from_slice(body);
    stream.write_all(&resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_root() -> PathBuf {
        let dir = std::env::temp_dir().join("web4server_test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn send_get(path: &str, addr: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        let req = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
        stream.write_all(req.as_bytes()).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        String::from_utf8_lossy(&buf).to_string()
    }

    fn serve_on_free_port(root: PathBuf) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let actual = listener.local_addr().unwrap().to_string();
        let r = root.clone();
        thread::spawn(move || {
            let _ = run_on_listener(listener, &r);
        });
        thread::sleep(std::time::Duration::from_millis(100));
        actual
    }

    #[test]
    fn test_resolve_path_rejects_dotdot() {
        let root = Path::new("/tmp");
        assert!(resolve_path(root, "/../etc/passwd").is_none());
        assert!(resolve_path(root, "/foo/../../etc/passwd").is_none());
        assert!(resolve_path(root, "/absolute/path").is_some());
        assert!(resolve_path(root, "/foo/bar").is_some());
    }

    #[test]
    fn test_content_type() {
        assert_eq!(content_type(Path::new("index.html")), "text/html; charset=utf-8");
        assert_eq!(content_type(Path::new("style.css")), "text/css; charset=utf-8");
        assert_eq!(content_type(Path::new("app.js")), "application/javascript; charset=utf-8");
        assert_eq!(content_type(Path::new("data.json")), "application/json");
        assert_eq!(content_type(Path::new("image.png")), "image/png");
        assert_eq!(content_type(Path::new("photo.jpeg")), "image/jpeg");
        assert_eq!(content_type(Path::new("file.unknown")), "application/octet-stream");
        assert_eq!(content_type(Path::new("file")), "application/octet-stream");
    }

    #[test]
    fn test_serve_file_success() {
        let root = tmp_root();
        fs::write(root.join("hello.txt"), b"Hello, World!").unwrap();
        let addr = serve_on_free_port(root);
        let resp = send_get("/hello.txt", &addr);
        assert!(resp.contains("200 OK"), "expected 200 OK, got: {resp}");
        assert!(resp.contains("Hello, World!"), "expected body, got: {resp}");
    }

    #[test]
    fn test_serve_404() {
        let root = tmp_root();
        let addr = serve_on_free_port(root);
        let resp = send_get("/nonexistent.txt", &addr);
        assert!(resp.contains("404 Not Found"), "expected 404, got: {resp}");
    }

    #[test]
    fn test_method_not_allowed() {
        let root = tmp_root();
        let addr = serve_on_free_port(root);

        let mut stream = TcpStream::connect(&addr).unwrap();
        let req = b"POST / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        stream.write_all(req).unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        let resp = String::from_utf8_lossy(&buf);
        assert!(resp.contains("405 Method Not Allowed"), "expected 405, got: {resp}");
    }

    #[test]
    fn test_serve_index_html() {
        let root = tmp_root();
        fs::write(root.join("index.html"), b"<h1>Index</h1>").unwrap();
        let addr = serve_on_free_port(root);
        let resp = send_get("/", &addr);
        assert!(resp.contains("200 OK"), "expected 200 OK, got: {resp}");
        assert!(resp.contains("<h1>Index</h1>"), "expected index body");
    }

    #[test]
    fn test_path_traversal_blocked() {
        let root = tmp_root();
        let addr = serve_on_free_port(root);
        let resp = send_get("/../Cargo.toml", &addr);
        assert!(resp.contains("404") || resp.contains("403"), "expected 403/404, got: {resp}");
    }
}

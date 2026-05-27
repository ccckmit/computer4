//! world4/render/server.rs
//! HTTP + WebSocket server for browser-based RL environment rendering.
//! Shares the same viewer.html with the TypeScript version.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const HTML: &str = include_str!("viewer.html");

pub struct RenderServer {
    latest_frame: Arc<Mutex<String>>,
}

impl RenderServer {
    pub fn start(port: u16) -> Self {
        Self::start_with_html(port, HTML)
    }

    pub fn start_with_html(port: u16, html: &'static str) -> Self {
        let latest = Arc::new(Mutex::new(String::new()));
        let latest_clone = latest.clone();

        thread::spawn(move || {
            let addr = format!("127.0.0.1:{}", port);
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[render] Failed to bind {}: {}", addr, e);
                    return;
                }
            };

            println!("[render] http://localhost:{}", port);

            let port_browser = port;
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(400));
                open_browser(port_browser);
            });

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let latest = latest_clone.clone();
                        thread::spawn(move || {
                            dispatch(stream, latest, html);
                        });
                    }
                    Err(_) => break,
                }
            }
        });

        RenderServer { latest_frame: latest }
    }

    pub fn send(&self, json: &str) {
        *self.latest_frame.lock().unwrap() = json.to_string();
    }
}

fn open_browser(port: u16) {
    let url = format!("http://localhost:{}", port);
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "linux") {
        "xdg-open"
    } else {
        return;
    };
    let _ = std::process::Command::new(cmd).arg(&url).spawn();
}

fn dispatch(mut stream: std::net::TcpStream, latest: Arc<Mutex<String>>, html: &str) {
    let mut buf = [0u8; 4096];
    let n = match stream.peek(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let request = String::from_utf8_lossy(&buf[..n]);

    if request.to_lowercase().contains("upgrade: websocket") {
        handle_ws(stream, latest);
    } else {
        serve_html(stream, html);
    }
}

fn serve_html(mut stream: std::net::TcpStream, html: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn handle_ws(stream: std::net::TcpStream, latest: Arc<Mutex<String>>) {
    use tungstenite::{accept, Message};

    let mut ws = match accept(stream) {
        Ok(ws) => ws,
        Err(_) => return,
    };

    loop {
        let frame = latest.lock().unwrap().clone();
        if !frame.is_empty() {
            if ws.send(Message::Text(frame)).is_err() {
                break;
            }
        }
        thread::sleep(Duration::from_millis(33));
    }
}
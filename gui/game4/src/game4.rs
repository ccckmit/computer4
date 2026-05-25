use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rusqlite::Connection;
use tungstenite::{accept, Message};

#[derive(Debug, Clone)]
pub struct ScoreEntry {
    pub player_score: usize,
    pub ai_score: usize,
    pub winner: String,
    pub rally: usize,
}

pub fn serve_static(mut stream: TcpStream, req: &str, game: &str) {
    let path = req.lines().next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");

    let stripped = &path[1..];
    let mime = match stripped.rsplit('.').next().unwrap_or("") {
        "html" | "" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        _ => "text/plain",
    };
    let rel_path = if stripped.is_empty() || stripped == "index.html" {
        format!("examples/{}/index.html", game)
    } else if stripped == "game4.js" || stripped == "game4.css" {
        stripped.to_string()
    } else if !stripped.contains('/') {
        format!("examples/{}/{}", game, stripped)
    } else {
        stripped.to_string()
    };

    let cwd = std::env::current_dir().unwrap();
    let full_path = cwd.join(&rel_path);

    match std::fs::read(&full_path) {
        Ok(body) => {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
                body.len(), mime
            );
            let mut resp = Vec::with_capacity(header.len() + body.len());
            resp.extend_from_slice(header.as_bytes());
            resp.extend_from_slice(&body);
            let _ = stream.write_all(&resp);
        }
        Err(_) => {
            let body = b"404 Not Found";
            let header = format!(
                "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let mut resp = Vec::with_capacity(header.len() + body.len());
            resp.extend_from_slice(header.as_bytes());
            resp.extend_from_slice(body);
            let _ = stream.write_all(&resp);
        }
    }
}

pub fn handle_ws(stream: TcpStream, db: Arc<Mutex<Connection>>) {
    let mut ws = match accept(stream) {
        Ok(ws) => ws,
        Err(_) => return,
    };

    let _ = ws.send(Message::Text(build_leaderboard(&db)));
    let _ = ws.get_mut().set_read_timeout(Some(Duration::from_millis(50)));

    loop {
        match ws.read() {
            Ok(Message::Text(txt)) => {
                if txt.contains("\"type\":\"score\"") && txt.contains("\"winner\"") {
                    if let Some(e) = parse_score(&txt) {
                        {
                            let d = db.lock().unwrap();
                            save_score(&d, &e);
                        }
                        let _ = ws.send(Message::Text(build_leaderboard(&db)));
                    }
                }
            }
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) => return,
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            _ => {}
        }

        thread::sleep(Duration::from_millis(10));
    }
}

pub fn parse_score(txt: &str) -> Option<ScoreEntry> {
    let ps = txt.split("\"player_score\":")
        .nth(1).and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())?.parse().ok()?;
    let as_ = txt.split("\"ai_score\":")
        .nth(1).and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())?.parse().ok()?;
    let winner = txt.split("\"winner\":\"")
        .nth(1).and_then(|s| s.split('"').next())?.to_string();
    let rally = txt.split("\"rally\":")
        .nth(1).and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())?.parse().ok()?;
    Some(ScoreEntry { player_score: ps, ai_score: as_, winner, rally })
}

pub fn save_score(db: &Connection, e: &ScoreEntry) {
    db.execute(
        "INSERT INTO scores (player_score, ai_score, winner, rally) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![e.player_score, e.ai_score, e.winner, e.rally],
    ).ok();
}

pub fn build_leaderboard(db: &Arc<Mutex<Connection>>) -> String {
    let db = db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT player_score, ai_score, winner, rally FROM scores ORDER BY id DESC LIMIT 10"
    ) {
        Ok(s) => s,
        Err(_) => return r#"{"type":"leaderboard","scores":[]}"#.to_string(),
    };

    let mut entries = String::new();
    let mut first = true;
    let rows = stmt.query_map([], |row| {
        Ok(ScoreEntry {
            player_score: row.get(0)?,
            ai_score: row.get(1)?,
            winner: row.get(2)?,
            rally: row.get(3)?,
        })
    });

    if let Ok(rows) = rows {
        for row in rows {
            if let Ok(e) = row {
                if !first { entries.push(','); }
                first = false;
                entries.push_str(&format!(
                    r#"{{"player_score":{},"ai_score":{},"winner":"{}","rally":{}}}"#,
                    e.player_score, e.ai_score, e.winner, e.rally
                ));
            }
        }
    }

    format!("{{\"type\":\"leaderboard\",\"scores\":[{}]}}", entries)
}

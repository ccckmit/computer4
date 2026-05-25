use std::fs;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use game4::*;
use rusqlite::Connection;

fn main() {
    fs::create_dir_all("db").ok();
    let db = Connection::open("db/assault.db").expect("open db/assault.db");
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS scores (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            player_score INTEGER NOT NULL,
            ai_score     INTEGER NOT NULL,
            winner       TEXT NOT NULL,
            rally        INTEGER NOT NULL,
            created_at   DATETIME DEFAULT CURRENT_TIMESTAMP
        );"
    ).expect("create table");

    let db = Arc::new(Mutex::new(db));
    let listener = TcpListener::bind("127.0.0.1:8081").unwrap();
    println!("Assault server at http://localhost:8081");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let db = db.clone();
                thread::spawn(|| {
                    let mut buf = [0u8; 4096];
                    let n = match s.peek(&mut buf) {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };
                    let req = String::from_utf8_lossy(&buf[..n]);
                    if req.to_lowercase().contains("upgrade: websocket") {
                        handle_ws(s, db);
                    } else {
                        serve_static(s, &req, "assault");
                    }
                });
            }
            Err(_) => break,
        }
    }
}

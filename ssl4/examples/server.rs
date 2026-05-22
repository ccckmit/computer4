//! 範例：啟動 TLS Echo 伺服器
//!   cargo run --example server

use ssl4::cert::generate_self_signed;
use ssl4::server::TlsServer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> ssl4::Result<()> {
    println!("🔐 產生自簽名憑證...");
    let pair = generate_self_signed("localhost", &["localhost", "127.0.0.1"])?;
    println!("✅ 憑證產生完成 (CN=localhost)\n");

    let server = TlsServer::new("127.0.0.1:8443", pair).await?;
    println!("🚀 監聽 {}，等待連線...", server.local_addr());
    println!("   執行 `cargo run --example client` 來測試\n");

    server.run(|mut stream, peer| async move {
        println!("🔗 新連線：{}", peer);
        let mut buf = vec![0u8; 4096];
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => { println!("🔌 斷線：{}", peer); break; }
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    println!("📨 [{}] {}", peer, text.trim());
                    let reply = format!("Echo: {}", text.trim());
                    if stream.write_all(reply.as_bytes()).await.is_err() { break; }
                }
                Err(e) => { eprintln!("❌ 讀取錯誤：{}", e); break; }
            }
        }
    }).await
}

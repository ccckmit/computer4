//! 範例：連線至 TLS Echo 伺服器
//!   cargo run --example client  (需先在另一終端機啟動 server)

use ssl4::client::{TlsClient, VerifyMode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> ssl4::Result<()> {
    println!("🔐 TLS 客戶端啟動");
    println!("⚠️  使用 DangerousNoVerify 模式（跳過憑證驗證，僅限測試）\n");

    let client = TlsClient::new("localhost", VerifyMode::DangerousNoVerify)?;
    let addr = "127.0.0.1:8443".parse().unwrap();

    println!("🔗 連線至 {} ...", addr);
    let mut stream = match client.connect(addr).await {
        Ok(s) => { println!("✅ TLS 握手成功！\n"); s }
        Err(e) => {
            eprintln!("❌ 連線失敗：{}", e);
            eprintln!("   請先執行：cargo run --example server");
            return Ok(());
        }
    };

    let messages = ["你好，TLS 伺服器！", "這是 Rust ssl4 套件", "再見！"];
    let mut buf = vec![0u8; 4096];

    for msg in &messages {
        println!("📤 傳送：{}", msg);
        stream.write_all(msg.as_bytes()).await?;
        stream.flush().await?;
        let n = stream.read(&mut buf).await?;
        println!("📩 收到：{}\n", String::from_utf8_lossy(&buf[..n]));
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    println!("✅ 通訊完成");
    Ok(())
}

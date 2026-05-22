# simple_ssl

一個用 Rust 寫的簡易 SSL/TLS 套件，基於 `rustls` + `tokio-rustls` 實作。

## 功能特色

- 🔐 **憑證模組** (`cert`)：自動產生自簽名憑證、從 PEM 載入
- 🖥️ **TLS 伺服器** (`server`)：非同步多連線 TLS 伺服器
- 📡 **TLS 客戶端** (`client`)：支援系統 CA、自訂 CA、跳過驗證三種模式

## 快速使用

### 產生憑證
```rust
use simple_ssl::cert::generate_self_signed;

let pair = generate_self_signed("localhost", &["localhost", "127.0.0.1"])?;
println!("{}", pair.cert_pem);
```

### 啟動 TLS 伺服器
```rust
use simple_ssl::{cert::generate_self_signed, server::TlsServer};

let pair = generate_self_signed("localhost", &["localhost"])?;
let server = TlsServer::new("0.0.0.0:8443", pair).await?;
server.run(|mut stream, peer| async move {
    // 處理連線 …
}).await?;
```

### 建立 TLS 客戶端
```rust
use simple_ssl::client::{TlsClient, VerifyMode};

// 正式環境：使用 Mozilla 根憑證
let client = TlsClient::new("example.com", VerifyMode::SystemRoots)?;

// 開發環境：跳過驗證
let client = TlsClient::new("localhost", VerifyMode::DangerousNoVerify)?;

let stream = client.connect("127.0.0.1:8443".parse()?).await?;
```

## 執行範例

```bash
# 終端機 1：啟動伺服器
cargo run --example server

# 終端機 2：連線客戶端
cargo run --example client

# 執行測試
cargo test
```

## 依賴套件

| 套件 | 說明 |
|------|------|
| rustls | 純 Rust TLS 實作 |
| tokio-rustls | tokio 非同步整合 |
| rcgen | 憑證產生 |
| webpki-roots | Mozilla 根憑證 |

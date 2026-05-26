# SSL/TLS

## 概述

SSL (Secure Sockets Layer) 與其後繼者 TLS (Transport Layer Security) 是為網路通訊提供加密與認證的安全協定。本專案的 `crypto/ssl4/` crate 使用 `rustls` 與 `tokio-rustls` 實作 SSL/TLS，用於保護網路通訊。

## SSL/TLS 的演進

| 版本 | 年 | 狀態 | 說明 |
|---|---|---|---|
| SSL 1.0 | 1994 | 未公開 | 設計缺陷，從未發布 |
| SSL 2.0 | 1995 | 廢棄 (2011) | 多個安全漏洞 |
| SSL 3.0 | 1996 | 廢棄 (2015) | POODLE 攻擊 |
| TLS 1.0 | 1999 | 廢棄 (2020) | SSL 3.0 的改進版 |
| TLS 1.1 | 2006 | 廢棄 (2020) | CBC 保護改進 |
| TLS 1.2 | 2008 | 現行 | SHA-256、AEAD、GCM |
| TLS 1.3 | 2018 | 現行 | 0-RTT、移除不安全演算法、大幅簡化交握 |

## TLS 交握 (Handshake)

```
用戶端                             伺服器
  │                                    │
  │──── ClientHello ─────────────────→│
  │   TLS 版本、密碼套件清單、          │
  │   隨機數、Session ID               │
  │                                    │
  │←─── ServerHello ─────────────────│
  │   選擇的版本、密碼套件、隨機數      │
  │                                    │
  │←─── Certificate ─────────────────│
  │   伺服器憑證鏈 (X.509)             │
  │                                    │
  │←─── ServerHelloDone ─────────────│
  │                                    │
  │──── ClientKeyExchange ───────────→│
  │   Pre-master secret (以公鑰加密)   │
  │                                    │
  │ 雙方各自計算 Master Secret          │
  │                                    │
  │──── ChangeCipherSpec ────────────→│
  │──── Finished ────────────────────→│
  │   從此開始加密通訊                  │
  │                                    │
  │←─── ChangeCipherSpec ────────────│
  │←─── Finished ────────────────────│
  │                                    │
  │══════ 加密通訊開始 ═══════════════│
  │   應用層資料 (HTTP, WebSocket...)  │
```

### TLS 1.3 簡化交握

TLS 1.3 將交握從 2-RTT 減少到 1-RTT（甚至 0-RTT）：

```
用戶端                             伺服器
  │                                    │
  │──── ClientHello (含 KeyShare) ──→│
  │   立即提供 Diffie-Hellman 參數     │
  │                                    │
  │←─── ServerHello + KeyShare ─────│
  │   + Certificate + Finished        │
  │                                    │
  │──── Finished ────────────────────→│
  │══════ 加密通訊開始 ═══════════════│
```

## 本專案的 ssl4 crate

`crypto/ssl4/` 使用 Rust 生態系的 `rustls`（純 Rust TLS 實作）：

### 依賴

```toml
# crypto/ssl4/Cargo.toml
[dependencies]
rustls = "0.20"
tokio-rustls = "0.23"
tokio = { version = "1", features = ["full"] }
webpki = "0.22"
webpki-roots = "0.22"
x509-parser = "0.14"
rcgen = "0.10"    # 憑證產生
```

### TLS 用戶端

```rust
use tokio_rustls::TlsConnector;
use rustls::ClientConfig;
use std::sync::Arc;

async fn tls_connect(domain: &str, port: u16) -> Result<()> {
    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let server_name = ServerName::try_from(domain)?;

    let tcp = TcpStream::connect((domain, port)).await?;
    let mut tls = connector.connect(server_name, tcp).await?;

    // 現在 tls 是加密連線
    tls.write_all(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n").await?;
    Ok(())
}
```

### TLS 伺服器

```rust
use tokio_rustls::TlsAcceptor;
use rustls::ServerConfig;

async fn tls_server() -> Result<()> {
    // 載入憑證與私鑰
    let certs = load_certs("cert.pem")?;
    let key = load_private_key("key.pem")?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("0.0.0.0:4433").await?;

    loop {
        let (tcp, _) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            let mut tls = acceptor.accept(tcp).await.unwrap();
            // 處理加密連線
        });
    }
}
```

## 憑證 (Certificate)

### X.509 憑證結構

```
Certificate:
    Version: 3
    Serial Number: 01:23:45...
    Issuer: CN=Example CA
    Validity:
        Not Before: 2026-01-01
        Not After : 2027-01-01
    Subject: CN=example.com
    Subject Public Key:
        Algorithm: RSA 2048 bits / ECDSA P-256
    Extensions:
        Subject Alternative Name: DNS:example.com
        Key Usage: Digital Signature, Key Encipherment
        Extended Key Usage: Server Authentication
    Signature Algorithm: sha256WithRSAEncryption
    Signature: ...
```

### 本專案的 keygen crate

`crypto/keygen/` CLI 工具產生 RSA/ECDSA 金鑰對與自簽憑證（自簽憑證的詳細說明見〈RSA〉專文）。

```sh
cd crypto/keygen
cargo run rsa 2048 --output key.pem --cert cert.pem
cargo run ecdsa P-256 --output ecdsa-key.pem --cert ecdsa-cert.pem
```

## rustls vs OpenSSL

| 特性 | rustls | OpenSSL |
|---|---|---|
| 語言 | Rust（純 Rust） | C |
| 記憶體安全 | ✓（Rust 保證） | 非記憶體安全 |
| 預設加密 | TLS 1.2/1.3 only | SSL 3.0+ |
| 預設密碼套件 | 僅 AEAD (GCM/Chacha20) | 含 CBC（需手動排除） |
| 平台依賴 | 無 | 系統 libssl |
| 二進位大小 | 較小 | 較大 |
| API 設計 | 現代 Rust 風格 | C API |

## SSL/TLS 在 browser4 的應用

```rust
// browser4 中 reqwest 使用 rustls-tls feature
// 自動處理 HTTPS 連線的 TLS 交握與驗證

// 注意：在開發環境中可使用自簽憑證
// reqwest 提供 danger_accept_invalid_certs(true) 跳過驗證
```

## 相關檔案

- `crypto/ssl4/src/lib.rs` — SSL/TLS 函式庫
- `crypto/ssl4/Cargo.toml` — rustls 依賴
- `crypto/keygen/src/main.rs` — 金鑰與憑證產生
- `web/browser4/src/main.rs` — 瀏覽器 HTTPS 支援
- `_wiki/rsa.md` — RSA 加密演算法
- `_wiki/cryptography.md` — 密碼學概述

## 參考資料

- TLS 1.3 (RFC 8446)：https://tools.ietf.org/html/rfc8446
- rustls 文件：https://docs.rs/rustls/
- SSL/TLS 運作原理：https://developer.mozilla.org/en-US/docs/Web/Security/Transport_Layer_Security
- Let's Encrypt：https://letsencrypt.org/

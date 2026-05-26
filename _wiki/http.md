# HTTP

## 概述

HTTP (HyperText Transfer Protocol) 是全球資訊網的基礎應用層協定，定義用戶端（瀏覽器）與伺服器之間的通訊格式。本專案的 `web/browser4/` 使用 `reqwest` crate 發送 HTTP 請求，`gui/game4/` 使用 WebSocket 而非 HTTP 進行即時遊戲通訊。

## HTTP 協定演進

| 版本 | 年 | 特性 |
|---|---|---|
| HTTP/0.9 | 1991 | 僅 GET 方法，無 header，純文字 |
| HTTP/1.0 | 1996 | 方法擴充 (GET/POST/HEAD)，header，狀態碼 |
| HTTP/1.1 | 1997 | 持久連線、chunked transfer、Host header、管線化 |
| HTTP/2 | 2015 | 多工串流、header 壓縮 (HPACK)、伺服器推送 |
| HTTP/3 | 2022 | QUIC (UDP-based)、0-RTT、更快的連線建立 |

## HTTP 請求-回應模型

```
用戶端 (瀏覽器)                    伺服器
      │                              │
      │────── HTTP 請求 ──────────→│
      │   GET /index.html HTTP/1.1    │
      │   Host: example.com          │
      │   User-Agent: browser4/0.1   │
      │   Accept: text/html          │
      │                              │
      │←───── HTTP 回應 ───────────│
      │   HTTP/1.1 200 OK            │
      │   Content-Type: text/html    │
      │   Content-Length: 1234       │
      │                              │
      │   <html>... (body)           │
      │                              │
```

### 請求方法

```rust
pub enum HttpMethod {
    GET,        // 取得資源
    POST,       // 提交資源
    PUT,        // 更新資源（完整取代）
    PATCH,      // 部分更新
    DELETE,     // 刪除資源
    HEAD,       // 僅取得 header
    OPTIONS,    // 查詢支援的方法
}
```

### 狀態碼分類

```rust
pub struct StatusCode(pub u16);

// 1xx Informational: 100 Continue, 101 Switching Protocols
// 2xx Success:       200 OK, 201 Created, 204 No Content
// 3xx Redirection:   301 Moved Permanently, 302 Found, 304 Not Modified
// 4xx Client Error:  400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found
// 5xx Server Error:  500 Internal Server Error, 502 Bad Gateway, 503 Service Unavailable
```

### Header

```rust
// 常用請求 header
pub const HOST: &str = "Host";
pub const USER_AGENT: &str = "User-Agent";
pub const ACCEPT: &str = "Accept";
pub const CONTENT_TYPE: &str = "Content-Type";
pub const AUTHORIZATION: &str = "Authorization";
pub const COOKIE: &str = "Cookie";
pub const REFERER: &str = "Referer";

// 常用回應 header
pub const CONTENT_LENGTH: &str = "Content-Length";
pub const CONTENT_TYPE_RESP: &str = "Content-Type";
pub const SET_COOKIE: &str = "Set-Cookie";
pub const CACHE_CONTROL: &str = "Cache-Control";
pub const LOCATION: &str = "Location";
```

## 本專案的 HTTP 使用

### browser4（reqwest）

```rust
#[tokio::main]
async fn fetch_url(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent("browser4/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let resp = client.get(url)
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await?;

    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.text().await?;

    Ok(body)
}
```

### browser5（reqwest 或檔案讀取）

```rust
fn load_url(&mut self, url: &str) {
    if url.starts_with("http://") || url.starts_with("https://") {
        // HTTP 請求（透過 reqwest）
        let html = fetch_http(&url);
        self.load_html(&html);
    } else {
        // 本地檔案讀取
        let html = std::fs::read_to_string(url).unwrap_or_default();
        self.load_html(&html);
    }
}
```

### HTTP over SSL/TLS

reqwest 預設使用 `rustls` 或 `native-tls` 處理 HTTPS：

```toml
# web/browser4/Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["rustls-tls"] }
```

`crypto/ssl4/` crate 則使用 `tokio-rustls` 實作自訂 SSL/TLS 層（見 SSL/TLS 專文）。

## HTTP 與 WebSocket

| 特性 | HTTP | WebSocket |
|---|---|---|
| 協定 | 請求-回應 | 全雙工訊息 |
| 連線 | 短連線 (HTTP/1.0) / 持久 (HTTP/1.1+) | 長連線 |
| 傳輸 | 文字 (二進位用 Base64) | 文字 + 二進位 (frame) |
| 標頭 | 每次請求都帶 header | 僅握手時 HTTP Upgrade |
| 應用 | REST API、網頁載入 | 即時遊戲、聊天、推播 |

game4 使用 WebSocket：

```rust
// game4 WebSocket 握手
// 從 HTTP Upgrade 開始
// GET /game HTTP/1.1
// Host: localhost:8080
// Upgrade: websocket
// Connection: Upgrade
// Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
// Sec-WebSocket-Version: 13
```

## 相關檔案

- `web/browser4/Cargo.toml` — reqwest 依賴配置
- `web/browser4/src/main.rs` — HTTP 請求實作
- `web/browser5/src/main.rs` — URL 載入（HTTP + 檔案）
- `crypto/ssl4/src/` — SSL/TLS 層實作
- `gui/game4/src/` — WebSocket 遊戲伺服器

## 參考資料

- HTTP/1.1 (RFC 7230-7235)：https://tools.ietf.org/html/rfc7230
- HTTP/2 (RFC 7540)：https://tools.ietf.org/html/rfc7540
- HTTP/3 (RFC 9114)：https://tools.ietf.org/html/rfc9114
- MDN HTTP：https://developer.mozilla.org/en-US/docs/Web/HTTP

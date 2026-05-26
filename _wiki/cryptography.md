# 密碼學 (Cryptography)

## 概述

密碼學是研究安全通訊技術的學科，涵蓋加密、解密、數位簽名、金鑰交換、憑證等領域。本專案的 `crypto/` 目錄包含兩個 crate：`ssl4`（SSL/TLS 協定實作）與 `keygen`（金鑰與憑證產生器）。

## 密碼學的基本概念

### 對稱加密 (Symmetric Encryption)

加密與解密使用相同金鑰：

```
明文 → [加密] → 密文 → [解密] → 明文
         ↑                    ↑
      同一金鑰              同一金鑰
```

- **優點：** 速度快、適合大量資料
- **缺點：** 金鑰配送問題
- **常見演算法：** AES、ChaCha20、DES（已不安全）
- **本專案使用：** 透過 rustls 使用 AES-GCM 或 ChaCha20-Poly1305

### 非對稱加密 (Asymmetric Encryption)

加密與解密使用不同金鑰（公鑰/私鑰對）：

```
明文 → [加密] → 密文 → [解密] → 明文
         ↑                    ↑
      公鑰                 私鑰
```

- **優點：** 無需預先共享金鑰
- **缺點：** 速度慢（比對稱加密慢數百倍）
- **常見演算法：** RSA、ECDSA、Ed25519
- **本專案使用：** RSA、ECDSA P-256/P-384（透過 keygen）

### 混合加密 (Hybrid Encryption)

實務上結合對稱與非對稱加密：

```
1. 以非對稱加密交換或產生臨時對稱金鑰
2. 以對稱加密保護實際資料
```

TLS 協定即採用此模式：先以非對稱密碼學完成交握，建立共享對稱金鑰（session key），再以對稱加密保護通訊內容。

## 雜湊函數 (Hash Function)

將任意長度的輸入映射到固定長度輸出（摘要 / digest）：

```
SHA-256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

特性：
- **單向性：** 無法從摘要反推輸入
- **碰撞抗性：** 很難找到兩個不同輸入產生相同摘要
- **雪崩效應：** 輸入微小變化導致輸出劇變

## 數位簽名 (Digital Signature)

使用私鑰簽署訊息，公鑰驗證：

```
簽署: hash(message) → 以私鑰加密 hash → 簽名
驗證: 以公鑰解密簽名 → 比對 hash(message)
```

提供：
- **身分驗證：** 確認簽署者身分
- **完整性：** 訊息未被篡改
- **不可否認性：** 簽署者無法否認

## 憑證 (Certificate)

憑證是將公鑰綁定到身分的數位文件，由憑證授權 (CA) 簽署：

```
Certificate:
    Data:
        Subject: CN = example.com
        Issuer: CN = Let's Encrypt
        Validity: 2026-01-01 ~ 2027-01-01
        Subject Public Key: (RSA 2048 bits)
    Signature Algorithm: SHA-256 with RSA
    Signature: ...
```

X.509 是最常用的憑證標準。

## TLS 協定 (Transport Layer Security)

TLS 是網際網路上最廣泛使用的安全通訊協定，位於 TCP 之上。

### 交握流程 (簡化)

```
Client                                  Server
  |                                        |
  |  ClientHello (支援的加密套件)          |
  |───────────────────────────────────────>|
  |                                        |
  |  ServerHello (選定的加密套件)          |
  |  ServerCertificate (伺服器憑證)        |
  |  ServerHelloDone                      |
  |<───────────────────────────────────────|
  |                                        |
  |  ClientKeyExchange (預先主金鑰)        |
  |  ChangeCipherSpec                      |
  |  Finished (加密的)                     |
  |───────────────────────────────────────>|
  |                                        |
  |  ChangeCipherSpec                      |
  |  Finished (加密的)                     |
  |<───────────────────────────────────────|
  |                                        |
  |  === 安全通道建立 ===                  |
  |  應用層資料 (AES-GCM 加密)             |
  |───────────────────────────────────────>|
```

### 本專案的 TLS 實作：ssl4

`crypto/ssl4/` 封裝了 rustls 與 tokio-rustls：

```rust
// ssl4 模組結構
pub mod cert;    // 憑證載入
pub mod server;  // TLS 伺服器
pub mod client;  // TLS 用戶端
pub mod error;   // 錯誤處理
```

```rust
// 範例：TLS 伺服器
use ssl4::cert::load_certificate;
use ssl4::server::TlsServer;

let cert = load_certificate("cert.pem", "key.pem")?;
let server = TlsServer::new(cert);
server.listen("0.0.0.0:4433")?;
```

## 密碼學在本專案的應用

### 金鑰產生 (keygen)

```
keygen key --key-type rsa --bits 4096 -o key.pem
keygen cert --key-type ecdsa-p256 --common-name "localhost" \
    --sans "127.0.0.1" -o cert.pem
keygen csr --key-type ecdsa-p384 --common-name "example.com" -o csr.pem
```

### TLS 通訊 (ssl4)

```sh
cd crypto/ssl4
./run.sh  # 啟動 TLS 伺服器範例
```

### 依賴的函式庫

| 函式庫 | 用途 |
|---|---|
| `rustls` | TLS 1.3 實作（純 Rust） |
| `tokio-rustls` | 非同步 TLS 包裝 |
| `rcgen` | X.509 憑證產生 |
| `webpki-roots` | Mozilla 根憑證集 |
| `rsa` | RSA 演算法 |
| `p256`, `p384` | ECDSA P-256/P-384 |
| `pem` | PEM 格式解析 |

## 安全的注意事項

1. **永遠使用經過驗證的加密函式庫** — 本專案使用 rustls（記憶體安全、純 Rust TLS 實作）
2. **金鑰長度建議：** RSA 2048+，ECDSA P-256+
3. **不要自己實作加密演算法** — 標準函式庫與經過審計的 crate 已提供可靠實作
4. **憑證驗證：** 確認憑證鏈、主機名稱、有效期限
5. **前向安全性 (Forward Secrecy)：** 使用 DHE 或 ECDHE 金鑰交換

## 常見密碼套件 (Cipher Suite)

```
TLS_AES_128_GCM_SHA256     // TLS 1.3 預設
TLS_AES_256_GCM_SHA384     // 更高安全性
TLS_CHACHA20_POLY1305_SHA256 // 無硬體 AES 加速時較快
```

格式說明：
- **金鑰交換：** ECDHE (橢圓曲線 Diffie-Hellman)
- **認證：** RSA 或 ECDSA
- **加密：** AES-GCM 或 ChaCha20-Poly1305
- **雜湊：** SHA-256 或 SHA-384

## 相關檔案

- `crypto/ssl4/src/lib.rs` — ssl4 入口
- `crypto/ssl4/src/cert.rs` — 憑證載入與驗證
- `crypto/ssl4/src/server.rs` — TLS 伺服器
- `crypto/ssl4/src/client.rs` — TLS 用戶端
- `crypto/keygen/src/main.rs` — 金鑰與憑證產生 CLI
- `crypto/ssl4/examples/` — 伺服器與用戶端範例

## 參考資料

- TLS 1.3 (RFC 8446)：https://tools.ietf.org/html/rfc8446
- X.509 (RFC 5280)：https://tools.ietf.org/html/rfc5280
- rustls 文件：https://docs.rs/rustls/
- Bruce Schneier, *Applied Cryptography*
- A. J. Menezes, *Handbook of Applied Cryptography*

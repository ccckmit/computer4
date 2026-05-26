# RSA

## 概述

RSA（Rivest-Shamir-Adleman）是目前最廣泛使用的非對稱加密演算法，由 Ron Rivest、Adi Shamir 和 Leonard Adleman 於 1977 年提出。RSA 的安全性基於大整數分解的難度：給定兩個大質數的乘積，在計算上難以反推出原始質數。

本專案的 `crypto/keygen/` crate 支援 RSA 金鑰產生（2048/4096 位元）。

## 數學基礎

### 歐拉定理

若 a 與 n 互質，則：
```
a^φ(n) ≡ 1 (mod n)
```
其中 φ(n) 是歐拉 totient 函數，表示小於等於 n 且與 n 互質的正整數個數。

### 模反元素

若 a 與 n 互質，則存在整數 b 使得：
```
a × b ≡ 1 (mod n)
```
b 稱為 a 在模 n 下的乘法反元素。

## RSA 金鑰產生

### 步驟

```
1. 選取兩個大質數 p 與 q（2048 位元 RSA 約需 1024 位元的質數）
2. 計算 n = p × q
3. 計算 φ(n) = (p-1)(q-1)
4. 選取公開指數 e（常用 65537 = 2^16 + 1）
5. 計算私密指數 d ≡ e^(-1) (mod φ(n))

公開金鑰: (e, n)
私密金鑰: (d, n)
```

### 為何 65537 常用作 e？

- 為質數：確保與 φ(n) 互質的機率高
- 漢明重量低：二進位表示為 `10000000000000001`（僅 2 個 1 位元），加速加密運算
- 夠大：抵抗低指數攻擊

## 加密與解密

### 加密
```
c = m^e mod n
```
m 為明文整數（需小於 n），c 為密文。

### 解密
```
m = c^d mod n
```
使用私密指數 d 還原明文。

### 正確性證明

由歐拉定理：
```
c^d ≡ (m^e)^d ≡ m^(ed) (mod n)
```
由於 ed ≡ 1 (mod φ(n))，存在整數 k 使 ed = 1 + kφ(n)：
```
m^(ed) ≡ m^(1 + kφ(n)) ≡ m × (m^φ(n))^k ≡ m × 1^k ≡ m (mod n)
```

## 本專案的 RSA 使用

`crypto/keygen/` crate 使用 `rcgen` 函式庫產生 RSA 金鑰與憑證：

```rust
// keygen CLI 命令
keygen key --key-type rsa --bits 2048 --output key.pem
keygen cert --key-type rsa --bits 2048 --common-name example.com --output cert.pem
```

金鑰儲存為 PEM 格式（Privacy-Enhanced Mail），使用 PKCS#8 編碼。

### 支援的金鑰類型

```rust
enum KeyType {
    Rsa,       // RSA 金鑰對（預設 2048 位元）
    EcdsaP256, // ECDSA P-256（橢圓曲線）
    EcdsaP384, // ECDSA P-384（橢圓曲線）
}
```

## RSA 的安全性

### 質因數分解攻擊

最直接的攻擊是分解 n。目前公開記錄：
- RSA-240 (795 bits) 於 2019 年被分解
- 建議使用至少 2048 位元的 n

### 其他攻擊

| 攻擊 | 原理 | 防禦 |
|---|---|---|
| 低指數攻擊 | e 太小且明文短時，m^e < n | 使用 e=65537 + 填充 |
| 共模攻擊 | 相同 n 不同 e | 每個使用者獨立的 n |
| 時序攻擊 | 測量解密時間 | 常數時間實現 |
| 選定密文攻擊 | 利用解密預言機 | OAEP 填充 |
| 中間人攻擊 | 攔截金鑰交換 | 憑證鏈驗證 |

### 填充方案 (Padding)

RSA 直接加密（教科書 RSA）不安全，需搭配填充：
- **PKCS#1 v1.5：** 舊版，有已知弱點
- **OAEP (Optimal Asymmetric Encryption Padding)：** 推薦使用，提供語意安全
- **PSS (Probabilistic Signature Scheme)：** 用於簽名

## RSA vs ECDSA

| 特性 | RSA | ECDSA |
|---|---|---|
| 安全性基礎 | 質因數分解 | 橢圓曲線離散對數 |
| 金鑰長度 (128-bit 安全) | 3072 bits | 256 bits |
| 加密速度 | 慢 | 較快 |
| 解密速度 | 慢 | 較快 |
| 簽名速度 | 慢 | 快 |
| 驗證速度 | 快 | 較慢 |
| 量子安全 | 否（Shor 演算法） | 否（Shor 演算法） |
| 廣泛支援 | 是（所有平台） | 是（現代平台） |

## 在本專案中的整合

### keygen crate

`crypto/keygen/` 提供三種子命令：

```sh
# 產生金鑰對
keygen key --key-type rsa --bits 4096 --output mykey.pem

# 產生自簽憑證
keygen cert --key-type rsa --bits 2048 --common-name "localhost" \
    --sans "127.0.0.1" --days 365

# 產生憑證簽署請求 (CSR)
keygen csr --key-type ecdsa-p256 --common-name "example.com" \
    --key mykey.pem --output mycsr.pem
```

### ssl4 crate

`crypto/ssl4/` 使用產生的金鑰與憑證建立 TLS 連線：

```rust
use ssl4::cert::load_certificate;
use ssl4::server::TlsServer;
use ssl4::client::TlsClient;

// 伺服器端（使用產生的金鑰與憑證）
let cert = load_certificate("cert.pem", "key.pem")?;
let server = TlsServer::new(cert);
server.listen("0.0.0.0:4433")?;

// 客戶端
let client = TlsClient::new();
client.connect("example.com:4433")?;
```

## 相關檔案

- `crypto/keygen/src/main.rs` — RSA/ECDSA 金鑰產生 CLI（208 行）
- `crypto/ssl4/src/lib.rs` — SSL/TLS 函式庫
- `crypto/ssl4/src/cert.rs` — 憑證載入與處理

## 參考資料

- R. Rivest, A. Shamir, L. Adleman, "A Method for Obtaining Digital Signatures and Public-Key Cryptosystems", 1978
- PKCS#1 v2.2 (RFC 8017)：https://tools.ietf.org/html/rfc8017
- PKCS#8 (RFC 5208)：https://tools.ietf.org/html/rfc5208
- NIST SP 800-57：金鑰管理建議

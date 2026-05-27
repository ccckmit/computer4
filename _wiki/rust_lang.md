# Rust 語言

## 概述

Rust 是一種系統程式語言，由 Mozilla 研究院開發（2010 年首次公開），專注於記憶體安全、併發安全與高效能，無需垃圾回收機 (GC)。本專案完全使用 Rust 實作，涵蓋從作業系統核心 (`#![no_std]`) 到圖形使用者介面 (eframe) 的各個層級。

## Rust 的所有權系統

所有權是 Rust 最獨特的特性，在編譯期保證記憶體安全：

### 三大規則

1. **每個值有一個所有者 (owner)**
2. **一次只能有一個可變參考或多個不可變參考**
3. **當所有者離開作用域，值被丟棄**

```rust
fn ownership_example() {
    let s = String::from("hello");  // s 是所有者
    let t = s;                      // 所有權轉移至 t（s 不再有效）
    // println!("{s}");             // 編譯錯誤！
    let u = &t;                     // u 是不可變參考
    println!("{u}");                // ✓
}   // t 離開作用域，記憶體自動釋放
```

### 借用 (Borrowing)

```rust
fn calc_length(s: &String) -> usize {  // 借入（不可變）
    s.len()
}

fn append_world(s: &mut String) {      // 可變借入
    s.push_str(" world");
}
```

### 所有權的意義

- 無需 GC：編譯器靜態分析決定何時釋放記憶體
- 無需手動 free：無 use-after-free 或 double-free 錯誤
- Send + Sync：執行緒安全由型別系統保證

## 本專案中的 Rust 使用模式

### no_std 核心

OS crate 使用 `#![no_std]`，無標準函式庫：

```rust
// os/mini-riscv-os/src/lib.rs
#![no_std]
#![no_main]

mod os_code;

#[panic_handler]
fn panic(_info: &core::panic::PanickInfo) -> ! {
    loop {}
}
```

### 靜態分配 (staticlib)

OS crate 編譯為靜態函式庫，在裸機環境使用：

```toml
# os/rvboard4/Cargo.toml
[lib]
crate-type = ["staticlib"]
```

### 無條件開啟 dead_code

部分大型 crate 使用 `#![allow(dead_code, unused)]`：

```rust
// database/db6/src/lib.rs
#![allow(dead_code, unused)]

// math4/src/lib.rs
#![allow(dead_code)]
```

### 錯誤處理

本專案普遍使用 Rust 的 `Result` 型別：

```rust
// 自訂錯誤型別
#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    ParseError(String),
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, Error>;

// 使用 `?` 運算子傳播錯誤
fn process_file(path: &str) -> Result<()> {
    let data = std::fs::read(path)?;   // 自動轉換錯誤
    let parsed = parse(&data)?;
    Ok(())
}
```

## Rust 的 Edition

本專案使用兩種版本：

| Edition | 差異 | 本專案的 crate |
|---|---|---|
| **2021** | Rust 1.56+，預設版本 | 大部分 crate（lli4、rv4、db6、ruhdl、...） |
| **2024** | Rust 1.85+，較新的特性 | sql4、btree、patricia-trie、redblacktree、lz4、aplayer4 + octopos |

Edition 2024 的新特性（相對於 2021）：
- 改進的 `impl Trait`
- `unsafe` 區塊內的改進
- `Never` 型別穩定化
- 更多 `const` 泛型

## 本專案的 Cargo 工作流程

### 獨立 crate 模型

不同於典型的 Cargo workspace 專案，本專案的每個 crate 完全獨立：

```sh
# 每個 crate 有自己的 Cargo.toml、Cargo.lock、target/
cd database/btree && cargo test
cd compiler/rv4    && cargo build
```

例外：`os/xv6-rust-octopus/` 與 `os/xv7-rust-octopus/` 使用 Cargo workspace。

### 無根目錄 rust-toolchain.toml

大多數 crate 使用系統預設的 Rust 工具鏈（stable）。只有 octopos 系列需要 nightly：

```toml
# os/xv6-rust-octopus/rust-toolchain.toml
[toolchain]
channel = "nightly-2024-09-01"
```

### 常見 trait 實作

```rust
// Default — 提供預設建構子
impl<K, V> Default for RedBlackTree<K, V> {
    fn default() -> Self { Self::new() }
}

// Clone — 深拷貝
impl<V: Clone> Clone for PatriciaTrie<V> { ... }

// Debug — 格式化輸出
impl<K: Debug, V: Debug> Debug for SwisstableMap<K, V> { ... }

// IntoIterator — for 迴圈支援
impl<'a, K, V> IntoIterator for &'a SwisstableMap<K, V> { ... }
```

## Rust 在裸機開發的重點

### 核心 crate

在 `#![no_std]` 環境中可用的 crate：

```rust
core   // 基本型別、迭代器、cell、原子操作
alloc  // Vec、Box、String、HashMap（需 allocator）
compiler_builtins // memcpy、memset 等內建函式
```

不可用：`std::fs`、`std::thread`、`std::net`、`std::io`

### 自訂 panic handler

裸機環境需自訂 panic 行為：

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // RISC-V 的 WFI (Wait for Interrupt) 指令
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}
```

## 本專案使用的外部 crate

本專案 45 份 `Cargo.toml` 共使用約 60 個不同的外部 crate，21 個子 crate 完全依賴標準函式庫（零外部依賴）。

### Web 與網路

| Crate | 用途 | 使用位置 |
|---|---|---|
| `axum` | 非同步 HTTP 框架 (db6 gRPC/REST) | database/db6 |
| `tokio` | 非同步執行器 | database/db6, crypto/ssl4 |
| `tonic` | gRPC 框架 | database/db6 |
| `reqwest` | HTTP 用戶端 | web/browser4, web/md4browser |
| `tungstenite` / `tokio-tungstenite` | WebSocket | gui/game4, database/db6 |
| `rustls` / `tokio-rustls` | TLS 加密 (純 Rust) | crypto/ssl4 |
| `webpki-roots` | TLS 根憑證 | crypto/ssl4 |
| `http` | HTTP 型別 | database/db6 |
| `futures` / `futures-util` | 非同步工具 | database/db6 |
| `prost` / `prost-types` / `prost-build` | protobuf 編解碼 | database/db6 |
| `bytes` | 位元組緩衝 | database/db6 |

### GUI 與圖形

| Crate | 用途 | 使用位置 |
|---|---|---|
| `eframe` | egui 框架 (原生視窗) | gui/win4, eda/ic4, web/browser4/5, web/md4browser |
| `egui_commonmark` | Markdown 渲染 (egui) | web/md4browser |
| `sdl2` | SDL2 繫結 (RV 模擬器) | os/rvboard4/simulator |
| `plotters` | 圖表繪製 | math4 |
| `dirs` | 系統目錄查詢 | gui/win4 |
| `chrono` | 日期時間 | gui/win4 |

### 序列化與儲存

| Crate | 用途 | 使用位置 |
|---|---|---|
| `serde` / `serde_json` / `serde_bytes` | 序列化框架 | database/db6, database/lsm, eda/ruspice |
| `bincode` | 二進位序列化 | database/db6, database/lsm |
| `zstd` | Zstandard 壓縮 | database/db6 |
| `memmap2` | 記憶體映射檔案 | database/db6 |
| `rusqlite` | SQLite 繫結 | gui/game4 |
| `tempfile` | 暫存檔案 (測試) | database/db6, database/lsm, compiler/objdump |

### 密碼學

| Crate | 用途 | 使用位置 |
|---|---|---|
| `rcgen` | TLS 憑證產生 | crypto/ssl4, crypto/keygen |
| `rsa` | RSA 演算法 | crypto/keygen |
| `p256` / `p384` | ECDSA (P-256/P-384) | crypto/keygen |
| `pkcs8` | PKCS#8 金鑰格式 | crypto/keygen |
| `pem` | PEM 格式編解碼 | crypto/keygen |

### 解析與語言

| Crate | 用途 | 使用位置 |
|---|---|---|
| `scraper` | HTML 解析 (CSS 選擇器) | web/browser4, web/browser5 |
| `ego-tree` | DOM 樹實作 (scraper 底層) | web/browser4, web/browser5 |
| `boa_engine` | JavaSript 引擎 (完整 ES2020+) | web/browser4 |
| `scroll` | 位元組偏移讀寫 | compiler/objdump |

### 媒體

| Crate | 用途 | 使用位置 |
|---|---|---|
| `image` | 圖片解碼 (PNG/JPEG/GIF/...) | math4, media/jpeg, web/browser5 |
| `rodio` | 音訊播放 (symphonia 後端) | media/aplayer4 |
| `ndarray` | N 維陣列 (NumPy-like) | math4 |
| `statrs` | 統計函式庫 | math4 |

### CLI 與終端機

| Crate | 用途 | 使用位置 |
|---|---|---|
| `clap` | 命令列參數解析 | compiler/objdump, crypto/keygen |
| `crossterm` | 終端機控制 (游標/色彩) | media/aplayer4, tool/vi4 |
| `assert_cmd` / `predicates` | CLI 測試工具 | compiler/objdump |

### 數學與模擬

| Crate | 用途 | 使用位置 |
|---|---|---|
| `rand` | 亂數產生 | math4, crypto/keygen, eda/ic4 |
| `nalgebra` | 線性代數 (矩陣/向量) | eda/ruspice |
| `approx` | 浮點數近似比較 (測試) | eda/ruspice |

### 底層與 OS

| Crate | 用途 | 使用位置 |
|---|---|---|
| `buddy-alloc` | buddy 記憶體分配器 (no_std) | xv6/xv7 kernel |
| `bytemuck` | 安全位元組轉型 | xv6/xv7 mkfs |
| `fastrand` | 快速亂數 | database/db6 |
| `thiserror` | 錯誤型別 derive macro | database/db6, database/lsm, compiler/objdump |

### 零外部依賴的 crate（僅 std）

```
database/sql4     database/btree     database/fts
database/lsm (*已列上方*)  database/swisstable
database/patricia-trie    database/redblacktree
database/inodefs   compiler/lli4     compiler/rustc4
compiler/rv4       web/xdom4         web/js4
media/mp3          media/mpeg1       eda/ruhdl
eda/verilog4       eda/synthesis     os/mini-riscv-os
os/rvboard4        tool/lz4
```
*\*lsm 使用 serde/bincode/thiserror，其餘 21 個 crate 完全零外部依賴。*

## 區域路徑依賴

部分 crate 依賴本專案內的其他 crate（而非 crates.io）：

```toml
# web/browser5/Cargo.toml
[dependencies]
xdom4 = { path = "../xdom4" }
js4 = { path = "../js4" }
```

```toml
# os/xv6-rust-octopus/user/Cargo.toml
kernel = { path = "../kernel" }
```

這意味著建置時需注意依賴順序，無法獨立編譯這些 crate。

## 相關檔案

- 根目錄多個 `Cargo.toml` — 各 crate 依賴配置
- `os/xv6-rust-octopus/rust-toolchain.toml` — nightly 工具鏈鎖定
- `os/mini-riscv-os/src/lib.rs` — no_std 核心範例

## 參考資料

- Rust 程式語言官方書籍：https://doc.rust-lang.org/book/
- Rust 標準函式庫文件：https://doc.rust-lang.org/std/
- The Rustonomicon：https://doc.rust-lang.org/nomicon/
- Rust 嵌入式開發：https://docs.rust-embedded.org/
- crates.io：https://crates.io/

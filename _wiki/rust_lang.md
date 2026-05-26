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

## 相關檔案

- 根目錄多個 `Cargo.toml` — 各 crate 依賴配置
- `os/xv6-rust-octopus/rust-toolchain.toml` — nightly 工具鏈鎖定
- `os/mini-riscv-os/src/lib.rs` — no_std 核心範例

## 參考資料

- Rust 程式語言官方書籍：https://doc.rust-lang.org/book/
- Rust 標準函式庫文件：https://doc.rust-lang.org/std/
- The Rustonomicon：https://doc.rust-lang.org/nomicon/
- Rust 嵌入式開發：https://docs.rust-embedded.org/

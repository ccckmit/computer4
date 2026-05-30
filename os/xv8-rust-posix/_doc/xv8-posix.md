# xv8 POSIX 架構規劃

> 目標：讓 `os/posix/tools` 的 source code 不需複製，直接在 xv8 上編譯執行。

---

## 1. 核心共識：Linux 的軟體堆疊

Linux 環境中，Rust 的 `std` 並非憑空出現，而是層層依賴：

```
┌─────────────────────────────────────┐
│ Rust std (Vec, String, Box, etc.)  │  ← 開發者直接使用的 API
├─────────────────────────────────────┤
│ alloc crate (Vec 背後)              │  ← 需要 #[global_allocator]
├─────────────────────────────────────┤
│ malloc (libc / system allocator)   │  ← 需要 brk/sbrk/mmap
├─────────────────────────────────────┤
│ Linux Kernel syscalls               │  ← brk(2), mmap(2), etc.
├─────────────────────────────────────┤
│ Linux Kernel (實體記憶體管理)        │  ← 頁面配置、swap
└─────────────────────────────────────┘
```

**Linux 提供給 `Vec`/`String` 的底層介面**：`brk`/`sbrk` syscalls（調整 heap 邊界）+ `mmap`（匿名映射）。

---

## 2. xv8 的對應架構

xv8 的目標：模擬 Linux 提供給 `alloc` 的底層介面：

```
┌─────────────────────────────────────┐
│ Rust std / alloc (Vec, String...)   │  ← 兩種 target 共用
├─────────────────────────────────────┤
│ xv8 user: extern crate alloc        │
│ 自動使用 kernel 提供的 global_alloc │
├─────────────────────────────────────┤
│ xv8 kernel: buddy-alloc             │  ← 單一 #[global_allocator]
│ (記憶體池，sbrk/mmap 的後端)         │
├─────────────────────────────────────┤
│ xv8 syscalls: sbrk, mmap, read...  │  ← user ↔ kernel IPC
└─────────────────────────────────────┘
```

### 2.1 Memory Model

- **Kernel (`kalloc.rs`)**: `buddy-alloc` 管理實體記憶體頁面，設置 `#[global_allocator]`。這是整個 binary（kernel + user 靜態連結）唯一的 allocator。
- **User `sbrk(n)`**: syscall 12，向 kernel 請求 `n` bytes 記憶體。kernel 從 `buddy-alloc` 的記憶體池取用，回傳指標。
- **User `mmap`**: syscall 34，支援 `MAP_ANONYMOUS | MAP_PRIVATE`，同樣由 buddy-alloc 支援。
- **為什麼只有一個 `global_allocator`？** xv8 是 static binary（kernel + user 共享同一個位址空間、同一個 linker script、同一個記憶體佈局）。Rust 的 `#[global_allocator]` 是 crate-level，不是 process-level。

### 2.2 `alloc` crate 如何接入

在 `user/src/lib.rs` 中：
```rust
#![no_std]
#![feature(alloc)]

extern crate alloc;   // 啟用 Vec/String/Box
```

此時 `alloc` 會自動使用 kernel 的 `#[global_allocator]`（即 `buddy-alloc`）。不需在 user 重新聲明 `global_allocator`。

### 2.3 `libposix` 的兩路設計

`libposix`（在 `os/posix/libposix/`）需要同時支援 Mac 和 xv8：

| 層面 | Mac (`#[cfg(unix)]`) | xv8 (`#[cfg(target_os = "none")]` |
|------|---------------------|-----------------------------------|
| 基礎 | `std` | `no_std` + `alloc` |
| 讀寫 | `std::io::Read/Write` + `libc` | syscall `read`/`write` (inline asm `ecall`) |
| 記憶體 | OS 自動 (malloc/brk) | kernel 的 `buddy-alloc` |
| args | `std::env::args()` → `Vec<String>` | `Args::from_stack()` → `Vec<String>` (需 `alloc`) |
| exit | `std::process::exit()` | `syscall::exit()` (ecall) |
| open | `libc::open` | `sys_open` (ecall) |

`libposix` 的模組結構：
```
libposix/
├── src/
│   ├── lib.rs          # 匯出 Read/Write/File/print/println/exit/args/stdin/stdout/stderr
│   ├── io.rs           # Read/Write traits + File + Stdin/Stdout/Stderr + syscall wrappers
│   └── fmt.rs          # (optional: format! wrapper)
```

---

## 3. Syscall 介面

xv8 已實作的 syscalls（與 `alloc`/`Vec` 相關）：

| 編號 | 名稱 | 用途 |
|------|------|------|
| 12 | `Sbrk` | 調整 heap 大小，給 `alloc` 用 |
| 34 | `Mmap` | 匿名記憶體映射，給 `alloc` 用（大塊配置） |
| 5 | `Read` | 讀取檔案/裝置 |
| 16 | `Write` | 寫入檔案/裝置 |
| 15 | `Open` | 開啟檔案 |
| 21 | `Close` | 關閉檔案描述符 |
| 10 | `Exit` | 終止行程 |

xv8 的 `sys_sbrk` 實作於 `kernel/src/sysproc.rs:117`，從 buddy-alloc 的記憶體池配置。

---

## 4. 實作記憶體分配器

### 4.1 xv8 kernel (`kalloc.rs`)

```rust
// kernel/src/kalloc.rs
use buddy_alloc::*;
static ALLOC: BuddyAlloc = BuddyAlloc::new(...);

#[global_allocator]
static ALLOCATOR: BuddyAlloc = ALLOC;
```

`buddy-alloc` 是 kernel 的內部分配器，管理預先設定的實體記憶體區塊。

### 4.2 `sys_sbrk` 到 `buddy-alloc` 的流程

```
user 程式: Vec::new()
  → alloc crate: allocate(Layout)
    → GlobalAlloc::alloc() [即 buddy-alloc]
      → 如果現有 pool 不夠，呼叫 sbrk(n)
        → syscall: sys_sbrk(n) [kernel/src/sysproc.rs]
          → buddy-alloc 配置 n bytes
          → 回傳指標到 user
```

---

## 5. `libposix` API 在 xv8 上的對應

| `libposix` API | Mac 實作 | xv8 實作 |
|----------------|---------|---------|
| `File::open(path)` | `libc::open` | `sys_open` (ecall) |
| `File::read(buf)` | `libc::read` | `sys_read` (ecall) |
| `File::write(buf)` | `libc::write` | `sys_write` (ecall) |
| `File::close()` | `libc::close` | `sys_close` (ecall) |
| `stdin()` / `stdout()` / `stderr()` | `std::io` singletons | `Stdin`/`Stdout`/`Stderr` singletons (fd 0/1/2) |
| `print(s)` / `println(s)` | `std::io::stdout().write_all()` | `sys_write(STDOUT_FD, ...)` (ecall) |
| `exit(code)` | `std::process::exit(code)` | `sys_exit(code)` (ecall, noreturn) |
| `args()` → `Vec<String>` | `std::env::args()` | `Args::from_stack()` + `alloc::vec![String]` |

---

## 6. 未來規劃

### 6.1 `libposix` 改造（進行中）

1. **統一包裝**: `libposix/io.rs` 用 `#[cfg(unix)]` / `#[cfg(target_os = "none")]` 分別實作
2. **工具改造**: `os/posix/tools/src/bin/` 下的工具逐漸把 `std::*` 替換為 `libposix::*`
3. **xv8 直接引用**: `xv8-rust-posix/user` 的 `Cargo.toml` 加 `libposix` path dependency

### 6.2 尚需實作

| 項目 | 說明 |
|------|------|
| `libposix::args()` on xv8 | 目前回 `Vec::new()`，需實作從 `Args` 到 `Vec<String>` 的轉換（需要 `String::from_utf8` + `alloc::vec!`）|
| `libposix::read_to_end()` | 方便方法，implement once `Vec` works |
| `File::open` 完整的 flags 支援 | 目前 `_flags: usize` 是 placeholder |
| `mmap` syscall wrapper | `libposix` 需要包裝 `mmap`，給 `alloc` 大塊配置使用 |

### 6.3 測試驗證

```sh
# Mac 上測試 libposix
cd os/posix/libposix && cargo build && cargo test

# xv8 上測試
cd os/xv8-rust-posix && ./test.sh
```

---

## 7. 關鍵檔案索引

| 檔案 | 用途 |
|------|------|
| `xv8-rust-posix/kernel/src/kalloc.rs` | `#[global_allocator]` + `buddy-alloc` 實作 |
| `xv8-rust-posix/kernel/src/sysproc.rs:117` | `sys_sbrk` syscall 實作 |
| `xv8-rust-posix/user/src/lib.rs` | user 入口，`extern crate alloc` |
| `xv8-rust-posix/user/src/args.rs` | `Args` struct，從 stack 讀取 argv/argc |
| `xv8-rust-posix/user/src/syscall.rs` | syscall wrappers（raw + safe） |
| `xv8-rust-posix/user/src/allocator.rs` | （已移除，kernel 的 global_allocator 共用）|
| `os/posix/libposix/src/io.rs` | Read/Write traits + syscall wrappers |
| `os/posix/libposix/src/lib.rs` | 匯出 public API |
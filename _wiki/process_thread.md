# 行程與執行緒 (Process & Thread)

## 概述

行程 (process) 與執行緒 (thread) 是作業系統中兩個基本的執行單元抽象。行程是資源分配的最小單位，執行緒是 CPU 排程的最小單位。本專案的 xv6-rust-octopus 支援行程管理，但採用簡化模型（無多執行緒）。Rust 的標準執行緒 (`std::thread`) 用於 db6、ssl4 等使用者層級的 crate。

## 行程 (Process)

### 行程的定義

行程是正在執行程式的實例，包含：
- **位址空間：** 程式碼、資料、堆疊、堆積
- **執行狀態：** 暫存器、程式計數器 (PC)
- **資源：** 開啟的檔案、訊號處理器、環境變數

### 行程控制區塊 (PCB, Process Control Block)

作業系統為每個行程維護的資料結構：

```rust
// xv6 的行程結構 (簡化)
pub struct Proc {
    pub state: ProcState,     // 行程狀態
    pub pid: i32,             // 行程 ID
    pub sz: u64,              // 行程大小（位元組）
    pub pagetable: *mut PageTable, // 頁表
    pub tf: *mut TrapFrame,   // 陷阱框架（保存暫存器）
    pub context: Context,     // 排程上下文
    pub parent: *mut Proc,    // 父行程
    pub kstack: u64,          // 核心堆疊位址
    pub ofile: [Option<Arc<File>>; NOFILE], // 開啟檔案表
    pub cwd: InodeRef,        // 目前工作目錄
}
```

### 行程生命週期

```
UNUSED → EMBRYO → RUNNABLE → RUNNING → ZOMBIE → UNUSED
                      ↑                     │
                      └─── SLEEPING ←───────┘
```

| 狀態 | 說明 |
|---|---|
| UNUSED | PCB 槽可用 |
| EMBRYO | `fork()` 剛建立，尚未就緒 |
| RUNNABLE | 可執行但未獲得 CPU |
| RUNNING | 正在 CPU 上執行 |
| SLEEPING | 等待資源或 I/O |
| ZOMBIE | 已結束，等待 `wait()` 回收 |

### fork-exec 模型

Unix 行程建立使用 fork-exec 兩階段：

```rust
// fork: 複製目前行程
let pid = fork();
if pid == 0 {
    // 子行程
    exec("ls", &["ls", "-l"]);  // 取代為 ls 程式
}

// fork 回傳值：
// 父行程收到子行程的 PID
// 子行程收到 0
// 錯誤時收到 -1
```

**Copy-on-Write (COW)：** fork 時不立即複製整個位址空間，而是共享頁面並設為唯讀。任一行程寫入時觸發缺頁，再複製該頁。xv6 使用 naive fork（立即複製）。

### 行程隔離

核心透過以下機制確保行程隔離：
1. **位址空間分離：** 每個行程擁有獨立頁表
2. **使用者/核心模式：** 使用者行程無法執行特權指令
3. **系統呼叫介面：** 使用者僅能透過系統呼叫請求核心服務
4. **檔案權限：** 檔案存取受權限控制

## 執行緒 (Thread)

### 執行緒的定義

執行緒是行程內的輕量級執行單元：
- **共享資源：** 位址空間、檔案描述器、訊號處理器（同一行程內）
- **私有資源：** 執行緒 ID (TID)、堆疊、暫存器、errno

### 使用者級 vs 核心級執行緒

| 特性 | 使用者級 (Green threads) | 核心級 (OS threads) |
|---|---|---|
| 管理層 | 使用者空間函式庫 | 作業系統核心 |
| 切換成本 | 極低（函式呼叫） | 高（需系統呼叫） |
| 多核心利用 | 無法平行執行 | 可平行執行 |
| 阻塞影響 | 一個執行緒阻塞全部 | 僅該執行緒阻塞 |
| 範例 | Go goroutines, Rust async | POSIX threads (pthreads) |

### Rust 的執行緒

本專案中的執行緒使用模式：

```rust
// db6: 非同步伺服器使用 tokio
use tokio::task;

tokio::spawn(async {
    // 非同步任務
});

// 典型多執行緒
use std::thread;

let handle = thread::spawn(move || {
    // 在新執行緒上執行
});

handle.join().unwrap();

// db6 中使用 std::sync 進行同步
use std::sync::{Arc, Mutex};
```

### 執行緒同步

執行緒因共享位址空間而需要同步機制，詳見〈競爭情況與互斥鎖〉。

## 本專案的行程模型比較

| 特性 | xv6 | mini-riscv-os | db6 (使用者層級) |
|---|---|---|---|
| 行程數 | 最多 64 | 無 | 不限 |
| 多執行緒 | 無 | 無 | 有 (std::thread) |
| 排程 | Round-robin | 協同式 | OS 排程 |
| 位址空間 | Sv39 分頁 | 無 MMU | OS 虛擬記憶體 |
| 行程通訊 | 管道 (pipe) | 無 | 訊息佇列 (msgq) |

## 上下文切換 (Context Switch)

見〈上下文切換〉專文。

## 相關檔案

- `os/xv6-rust-octopus/kernel/src/proc.rs` — xv6 行程管理
- `os/xv6-rust-octopus/kernel/src/swtch.rs` — 排程上下文切換
- `os/xv6-rust-octopus/kernel/src/syscall.rs` — 系統呼叫分發
- `database/db6/src/msgq/` — 使用者層級訊息佇列（類 IPC）

## 參考資料

- Andrew S. Tanenbaum, *Modern Operating Systems*, Chapter 2: Processes and Threads
- xv6 教材 Chapter 5: Scheduling
- The Rustonomicon: Concurrency

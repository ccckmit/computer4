# 作業系統 (Operating System)

## 概述

作業系統是管理電腦硬體與軟體資源的系統軟體，提供行程管理、記憶體管理、檔案系統、裝置驅動、網路通訊等核心功能。本專案包含多個作業系統實作：`mini-riscv-os`（極簡核心）、`rvboard4`（RISC-V BSP）、`xv6-rust-octopus`（xv6 移植版）、`xv7-rust-octopus`（具網路功能的 xv6）。

## 作業系統的核心功能

### 1. 行程管理 (Process Management)

- 行程建立與終止
- 排程 (scheduling)：決定哪個行程獲得 CPU
- 同步 (synchronization)：防止競爭情況
- 行程間通訊 (IPC)

### 2. 記憶體管理 (Memory Management)

- 虛擬記憶體：每個行程擁有獨立的位址空間
- 分頁 (paging)：將虛擬位址映射到實體位址
- 交換 (swapping)：在記憶體與磁碟間移動資料

### 3. 檔案系統 (File System)

- 檔案的建立、讀寫、刪除
- 目錄結構
- 權限管理
- 持久化儲存

### 4. 裝置管理 (Device Management)

- 統一 I/O 介面
- 中斷處理
- DMA (Direct Memory Access)

### 5. 網路通訊 (Networking)

- 協定堆疊 (TCP/IP)
- 網路介面管理

## 特權模式 (Privilege Levels)

RISC-V 定義三種特權等級：

```
Machine (M) — 最高權限，韌體/RustSBI
    ↑
Supervisor (S) — 作業系統核心 (kernel)
    ↑
User (U) — 最低權限，應用程式
```

- Machine mode：處理開機、中斷、異常
- Supervisor mode：執行核心程式碼，可存取頁表
- User mode：執行使用者程式，受限的指令與記憶體存取

本專案的 OS crate 主要在 Supervisor mode 下運作。

## 中斷與例外 (Interrupts & Exceptions)

| 類型 | 來源 | 範例 |
|---|---|---|
| 中斷 (Interrupt) | 外部裝置 | 計時器、鍵盤 |
| 例外 (Exception) | 指令執行錯誤 | 除零、無效指令 |
| 系統呼叫 (Trap) | 軟體請求 | ecall 指令 |

RISC-V 的中斷控制器 (PLIC, Platform-Level Interrupt Controller) 負責管理中斷優先級與分發。

## 系統呼叫 (System Calls)

使用者程式與核心之間的介面：

```
應用程式 (User mode)
    │  write(fd, buf, n)
    │  ↓ ecall
    ▼
核心 (Supervisor mode)
    │  處理系統呼叫
    │  存取硬體
    │  ↓ mret/sret
    ▼
應用程式 (User mode)
```

xv6/xv7 支援的典型系統呼叫：

```c
int fork()           // 建立行程
int exit(int status) // 終止行程
int wait(int* status)// 等待子行程
int kill(int pid)    // 終止指定行程
int read(int fd, char* buf, int n)
int write(int fd, char* buf, int n)
int open(char* file, int flags)
int close(int fd)
int exec(char* file, char* argv[])
```

## 本專案的作業系統實作

### mini-riscv-os

極簡 RISC-V 作業系統核心，適合教學。

```
特點：
- #![no_std] Rust staticlib
- 組合語言啟動 (start.s, sys.s)
- 自訂鏈結腳本 (os.ld)
- QEMU virt 開發板
- 基本 I/O 與中斷處理
```

### xv6-rust-octopus

經典 xv6 教學 OS 的 Rust 移植。

```
架構：
┌─────────────────────────────────────┐
│  使用者程式 (sh, cat, ls, echo...)  │
├─────────────────────────────────────┤
│  系統呼叫介面 (ecall → syscall)      │
├─────────────────────────────────────┤
│  核心服務                            │
│  行程管理  記憶體管理  檔案系統        │
│  排程器   虛擬記憶體    inode          │
├─────────────────────────────────────┤
│  裝置驅動 (UART、計時器、virtio)      │
├─────────────────────────────────────┤
│  RustSBI (M mode bootloader)        │
└─────────────────────────────────────┘
```

### xv7-rust-octopus

xv6 的增強版，加入網路支援：
- TAP 網路設備
- UDP 通訊協定
- `setup_net.sh` 腳本設定虛擬網路

## 行程 vs 執行緒

| 特性 | 行程 (Process) | 執行緒 (Thread) |
|---|---|---|
| 位址空間 | 獨立 | 共享 |
| 資源開銷 | 大（PCB、記憶體映射） | 小（僅堆疊與暫存器） |
| 切換成本 | 高（TLB 刷新） | 低 |
| 通訊方式 | IPC（管道、訊息佇列） | 共享記憶體 |
| 隔離性 | 高（彼此不受影響） | 低（一個 thread 崩潰影響全部） |
| 建立速度 | 慢 | 快 |

## xv6 行程狀態

```
   ┌──────────┐
   │   UNUSED │  ← 可用 PCB 槽
   └────┬─────┘
        │ fork()
   ┌────▼─────┐
   │ USED     │
   │ EMBRYO   │  ← 正在初始化
   └────┬─────┘
        │ 資源準備就緒
   ┌────▼─────┐     schedule()    ┌──────────┐
   │ RUNNABLE │─────────────────> │ RUNNING  │
   └────┬─────┘                   └────┬─────┘
        │ 等待 I/O/資源                │ I/O 完成或取得資源
   ┌────▼─────┐                        │
   │ SLEEPING │<───────────────────────┘
   └──────────┘
        │ exit()
   ┌────▼─────┐
   │ ZOMBIE   │  ← 等待父行程 wait()
   └──────────┘
```

## 相關檔案

- `os/mini-riscv-os/src/lib.rs` — 最小核心
- `os/mini-riscv-os/start.s` — 組合語言啟動
- `os/mini-riscv-os/os.ld` — 鏈結腳本
- `os/xv6-rust-octopus/kernel/src/` — xv6 核心原始碼
- `os/xv6-rust-octopus/user/` — 使用者程式
- `os/xv7-rust-octopus/setup_net.sh` — 網路設定腳本

## 參考資料

- Andrew S. Tanenbaum, *Modern Operating Systems*
- xv6 原始碼 (MIT)：https://pdos.csail.mit.edu/6.828/
- RISC-V 特權架構規格：https://riscv.org/technical/specifications/
- RustSBI：https://github.com/rustsbi/rustsbi

# 排程 (Scheduling)

## 概述

排程是作業系統決定哪個行程或執行緒獲得 CPU 使用權的機制。排程器 (scheduler) 在行程之間切換，以達到最大化吞吐量、最小化延遲、確保公平性等目標。本專案的 xv6-rust-octopus 採用簡化的輪詢排程 (round-robin scheduling)。

## 排程的目標

- **公平性：** 每個行程獲得合理的 CPU 時間
- **效率：** 最大化 CPU 使用率（減少閒置）
- **回應時間：** 減少互動式行程的延遲
- **吞吐量：** 單位時間內完成的行程數
- **避免飢餓 (starvation)：** 確保所有行程最終都能執行

## 排程演算法

### 1. 先來先服務 (FCFS, First-Come First-Served)

```
行程 A (10ms) | 行程 B (5ms) | 行程 C (3ms)
A: 0-10ms, B: 10-15ms, C: 15-18ms
```

- **優點：** 簡單、公平
- **缺點：** 長行程阻塞短行程（護航效應, convoy effect）

### 2. 最短工作優先 (SJF, Shortest Job First)

```
行程 C (3ms) | 行程 B (5ms) | 行程 A (10ms)
C: 0-3ms, B: 3-8ms, A: 8-18ms
```

- **優點：** 最小化平均等待時間
- **缺點：** 無法預知執行時間、長行程可能飢餓

### 3. 優先權排程 (Priority Scheduling)

每個行程有優先權值，排程器選擇最高優先權的行程執行。

- **優點：** 可區分重要與不重要行程
- **缺點：** 低優先權行程可能永不執行（需老化機制 aging）

### 4. 輪詢排程 (Round-Robin, RR)

每個行程獲得固定的時間量子 (time quantum / time slice)，時間到就強制切換：

```
時間量子 = 2ms
A(0-2) → B(2-4) → C(4-6) → A(6-8) → B(8-10) → C(10-12) → A(12-14)
```

- **優點：** 公平、回應時間佳
- **缺點：** 量子太小 → 過多上下文切換；太大 → 接近 FCFS

### 5. 多層回饋佇列 (MLFQ, Multi-Level Feedback Queue)

多個佇列，每個具有不同的優先權與時間量子：
```
佇列 0 (優先權高, 量子 1ms)
佇列 1 (優先權中, 量子 4ms)
佇列 2 (優先權低, 量子 16ms)
```

行程用完量子就降級；互動式行程（常因 I/O 主動放棄 CPU）保持高優先權。

- **本專案：** xv6 使用此演算法

## xv6 的排程實作

xv6 使用簡化的輪詢排程，巡迴所有 RUNNABLE 行程：

```c
// xv6 排程器核心 (簡化)
void scheduler(void) {
    struct proc *p;
    for (;;) {
        // 關閉中斷
        // 巡迴所有行程
        for (p = process_table; p < &process_table[NPROC]; p++) {
            if (p->state == RUNNABLE) {
                // 執行行程切換
                p->state = RUNNING;
                switch_to(p);
                // 行程歸還控制權後
                p->state = p->state; // 由行程本身設定
            }
        }
    }
}
```

### 時脈中斷 (Timer Interrupt)

排程器由時脈中斷驅動：

```c
// 每秒 100 次（x86 版 xv6）或依 QEMU 設定
void timer_interrupt() {
    // 保存上下文
    // 遞增 ticks 計數器
    // 呼叫 yield() 強制行程自願放棄 CPU
    yield();
}
```

### 上下文切換 (Context Switch)

```
行程 A 執行中
    │  中斷或 yield()
    ├─ 保存 A 的暫存器到 A 的 PCB
    ├─ 載入 B 的暫存器從 B 的 PCB
    ├─ 設定 STVEC (中斷向量表)
    ├─ 切換頁表 (satp 暫存器)
    └─ 恢復 B 的執行
行程 B 執行中
```

### 排程時機

1. **主動放棄：** `yield()` 或 `sleep()` 系統呼叫
2. **被動搶占：** 時脈中斷觸發
3. **I/O 等待：** 行程等待裝置或資源
4. **行程結束：** `exit()`

## 排程的效能指標

```
等待時間 = 開始執行時間 - 抵達時間
周轉時間 = 完成時間 - 抵達時間
回應時間 = 第一次執行時間 - 抵達時間
CPU 使用率 = CPU 忙碌時間 / 總時間
吞吐量 = 完成的行程數 / 單位時間
```

### 範例比較

假設行程 A(10ms)、B(5ms)、C(3ms)，同時抵達：

| 演算法 | 平均周轉時間 | 平均等待時間 |
|---|---|---|
| FCFS (A→B→C) | (10+15+18)/3 = 14.3ms | (0+10+15)/3 = 8.3ms |
| SJF (C→B→A) | (3+8+18)/3 = 9.7ms | (0+3+8)/3 = 3.7ms |
| RR (量子=2ms) | 同 FCFS | 更長但更公平 |

## RISC-V 的排程相關暫存器

```
mstatus: M mode 狀態（中斷啟用/關閉）
mie:    M mode 中斷啟用（設定哪些中斷可觸發）
mip:    M mode 中斷等待（哪些中斷已觸發）
stvec:  S mode 中斷向量表位址
sepc:   S mode 例外 PC
scause: S mode 例外原因
stval:  S mode 例外值（錯誤位址等）
satp:   S mode 位址轉換與保護（頁表基底）
```

## 相關檔案

- `os/xv6-rust-octopus/kernel/src/proc.rs` — 行程管理與排程
- `os/xv6-rust-octopus/kernel/src/trap.rs` — 中斷/例外處理
- `os/xv6-rust-octopus/kernel/src/swtch.rs` — 上下文切換
- `os/xv6-rust-octopus/kernel/src/sleeplock.rs` — 睡眠鎖

## 參考資料

- Andrew S. Tanenbaum, *Modern Operating Systems*, Chapter 2: Processes and Threads
- xv6 排程實作：https://pdos.csail.mit.edu/6.828/
- RISC-V 特權架構規格，Chapter 3: Supervisor Level

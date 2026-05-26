# 競爭情況與互斥鎖 (Race Condition & Mutex)

## 概述

競爭情況 (race condition) 是並行程式設計中最常見且最難除錯的問題。當多個執行單元同時存取共享資源，且結果取決於執行順序時，即發生競爭情況。互斥鎖 (mutex) 是最基本的同步原語，用於保護共享資源，確保同一時間只有一個執行單元可存取該資源。

## 競爭情況 (Race Condition)

### 定義

競爭情況發生在系統的行為取決於無法控制的事件順序時。在並行程式設計中，這通常指多個執行緒同時讀寫共享變數，而最終結果不確定。

### 經典問題：銀行帳戶

```rust
// 兩個執行緒同時執行此程式碼
fn withdraw(account: &mut i32, amount: i32) {
    if *account >= amount {        // 檢查餘額
        *account -= amount;        // 扣款
    }
}

// 初始餘額: 100
// 執行緒 A 提款 80: 檢查餘額 (100 >= 80) → true
// 執行緒 B 提款 80: 檢查餘額 (100 >= 80) → true (A 尚未完成扣款!)
// 執行緒 A 扣款: account = 100 - 80 = 20
// 執行緒 B 扣款: account = 20 - 80 = -60 ← 餘額為負!
```

### 臨界區段 (Critical Section)

臨界區段是存取共享資源的程式碼區塊，需確保不超過一個執行緒在臨界區段內：

```
執行緒 A:          執行緒 B:
[進入臨界區段]      [等待]
account -= 80       ...
[離開臨界區段]      [進入臨界區段]
                    account -= 80
                    [離開臨界區段]
```

## 互斥鎖 (Mutex)

Mutex (Mutual Exclusion) 是最基本的同步機制，確保一次只有一個執行緒能進入臨界區段。

### 基本操作

```rust
// Rust std 的 Mutex
use std::sync::Mutex;

let counter = Mutex::new(0);

// lock() 回傳 MutexGuard（離開作用域時自動解鎖）
{
    let mut guard = counter.lock().unwrap();
    *guard += 1;
} // 在此自動解鎖
```

### Mutex 的實作原理

```
Mutex 內部包含:
  1. 一個狀態變數 (locked/unlocked)
  2. 一個等待佇列 (blocked threads)

lock():
  if unlocked → 設為 locked, 繼續執行
  if locked   → 將目前執行緒加入等待佇列並阻塞

unlock():
  設為 unlocked
  喚醒等待佇列中的一個執行緒
```

### 硬體支援

Mutex 需要硬體層級的原子操作：

**Test-and-Set (TAS)：**
```rust
// 原子操作：讀取值、設定為 1、回傳舊值
fn test_and_set(lock: &AtomicBool) -> bool {
    lock.swap(true, Ordering::Acquire)  // 原子交換
}

fn lock(lock: &AtomicBool) {
    while test_and_set(lock) {
        // busy waiting (spin)
    }
}

fn unlock(lock: &AtomicBool) {
    lock.store(false, Ordering::Release);
}
```

**Compare-and-Swap (CAS)：**
```rust
fn compare_and_swap(ptr: &AtomicU32, expected: u32, new: u32) -> u32 {
    ptr.compare_exchange(expected, new, Ordering::AcqRel, Ordering::Relaxed)
        .unwrap_or_else(|x| x)
}
```

RISC-V 提供 `lr.w` (Load-Reserved) 與 `sc.w` (Store-Conditional) 指令實現原子操作。

### 自旋鎖 vs 睡眠鎖

| 特性 | 自旋鎖 (Spinlock) | 睡眠鎖 (Sleeplock) |
|---|---|---|
| 等待行為 | CPU 忙轉 (busy-wait) | 執行緒睡眠、讓出 CPU |
| 適合場景 | 快速操作 (數十指令) | 長時間等待 (I/O) |
| 多核心 | 適合（另一核心快速釋放） | 適合 |
| 單核心 | 危險（可能死鎖） | 安全 |
| xv6 使用 | 中斷處理、短暫鎖 | 檔案系統、I/O |

xv6 實作兩種鎖：

```c
// 自旋鎖 (spinlock.h)
struct spinlock {
    uint locked;       // 鎖定狀態
    char *name;        // 鎖名稱
    struct cpu *cpu;   // 持有鎖的 CPU
};

// 睡眠鎖 (sleeplock.h)
struct sleeplock {
    uint locked;
    struct spinlock lk;  // 保護 sleeplock 的自旋鎖
    char *name;
};
```

## 其他同步機制

### 信號量 (Semaphore)

由 Dijkstra 提出的計數式同步機制：

```rust
struct Semaphore {
    count: AtomicI32,
    wait_queue: Queue,
}

impl Semaphore {
    fn wait(&self) {   // P (proberen = test)
        self.count.fetch_sub(1);
        if self.count < 0 { block(); }
    }

    fn signal(&self) { // V (verhogen = increment)
        self.count.fetch_add(1);
        if self.count <= 0 { wake(); }
    }
}
```

二元信號量 (= 最大計數為 1) 等同於 Mutex。

### 條件變數 (Condition Variable)

與 Mutex 搭配使用，允許執行緒等待特定條件成立：

```rust
let pair = Arc::new((Mutex::new(false), Condvar::new()));
let (lock, cvar) = &*pair;

// 等待執行緒
let mut ready = lock.lock().unwrap();
while !*ready {
    ready = cvar.wait(ready).unwrap();  // 原子解鎖 + 阻塞
}

// 喚醒執行緒
let mut ready = lock.lock().unwrap();
*ready = true;
cvar.notify_one();
```

### 讀寫鎖 (RwLock)

允許多個讀取者（不互斥）或單一寫入者（互斥）：

```rust
use std::sync::RwLock;

let data = RwLock::new(vec![1, 2, 3]);

// 多執行緒可同時讀取
let r1 = data.read().unwrap();
let r2 = data.read().unwrap();

// 寫入需獨佔（阻塞所有讀取）
let mut w = data.write().unwrap();
w.push(4);
```

### 屏障 (Barrier)

等待所有執行緒到達同一點後才繼續：

```rust
use std::sync::Barrier;

let barrier = Arc::new(Barrier::new(5)); // 等待 5 個執行緒
// 每個執行緒呼叫 barrier.wait()
```

## xv6 的鎖實作

xv6 使用自旋鎖保護核心資料結構：

```c
// xv6 自旋鎖實作 (簡化)
void acquire(struct spinlock *lk) {
    push_off();  // 關閉中斷
    while (__sync_lock_test_and_set(&lk->locked, 1)) {
        // spin
    }
    __sync_synchronize();
    lk->cpu = mycpu();
}

void release(struct spinlock *lk) {
    lk->cpu = 0;
    __sync_synchronize();
    __sync_lock_release(&lk->locked);
    pop_off();   // 恢復中斷
}
```

為什麼在自旋鎖中關閉中斷？
- 避免死鎖：若持鎖時發生中斷，而中斷處理程序嘗試取得同一把鎖
- 單核心系統中必備（否則 spin 永遠不成功）

## 常見問題

### 死鎖 (Deadlock)

四個必要條件（Coffman 條件）：
1. **互斥：** 資源不可共享
2. **持有並等待：** 持有一個資源的同時等待另一個
3. **不可搶占：** 資源不能被強制釋放
4. **循環等待：** 各執行緒形成循環等待

```
執行緒 A: lock(X) → lock(Y)  // 等待 Y
執行緒 B: lock(Y) → lock(X)  // 等待 X
// 死鎖！
```

預防：固定鎖獲取順序（總是先 X 後 Y）。

### 優先權反轉 (Priority Inversion)

低優先權執行緒持有鎖，高優先權執行緒等待：
- 中優先權執行緒搶占低優先權執行緒 → 高優先權執行緒無法取得鎖
- 解決方案：優先權繼承 (priority inheritance)

### 資料競爭 (Data Race)

Rust 的型別系統透過 `Send` 與 `Sync` trait 在編譯期防止資料競爭：
- `Send`：型別可安全地線上程間轉移所有權
- `Sync`：型別可安全地線上程間共享參考 (`&T` 可共享 ⇔ `T: Sync`)

```rust
fn is_send<T: Send>() {}
fn is_sync<T: Sync>() {}

is_send::<Mutex<i32>>();  // ✓
is_sync::<Mutex<i32>>();  // ✓
is_send::<Rc<i32>>();     // ✗ Rc 非 Send
is_sync::<RefCell<i32>>(); // ✗ RefCell 非 Sync
```

## 相關檔案

- `os/xv6-rust-octopus/kernel/src/spinlock.rs` — xv6 自旋鎖
- `os/xv6-rust-octopus/kernel/src/sleeplock.rs` — xv6 睡眠鎖
- `database/db6/src/engine/lsm.rs` — Rust RwLock 的使用範例
- `database/lsm/src/lsm/engine.rs` — LSM 引擎的 RwLock 保護

## 參考資料

- E. W. Dijkstra, "Cooperating sequential processes", 1965
- xv6 教材 Chapter 4: Locking
- Rust 的 Send 與 Sync：https://doc.rust-lang.org/nomicon/send-and-sync.html

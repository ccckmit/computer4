# LSM-Tree

## 概述

LSM-Tree (Log-Structured Merge-Tree) 是一種專為高寫入吞吐量設計的資料結構，由 Patrick O'Neil 等人於 1996 年提出。與傳統 B-Tree 不同，LSM-Tree 將寫入操作先快取在記憶體中，再批次合併到磁碟，將隨機寫入轉換為循序寫入，顯著提升寫入效能。

## 在本專案中的角色

本專案包含兩個 LSM-Tree 實作：
1. `database/lsm/` — 獨立 LSM-Tree 引擎 crate
2. `database/db6/` — 旗艦資料庫 db6 中的 LSM 引擎（透過 `StorageEngine` trait 整合）

## 架構

### 分層結構

LSM-Tree 由三層組成：

```
┌─────────────────────────┐
│      MemTable           │  ← Level 0 (記憶體，可寫入)
│   (BTreeMap in RAM)     │
├─────────────────────────┤
│     SSTable #0          │  ← Level 1 (磁碟，唯讀)
│     SSTable #1          │
│     SSTable #2          │
├─────────────────────────┤
│     SSTable #3          │  ← Level 2 (磁碟，唯讀)
│     SSTable #4          │
│     ... (more)          │
└─────────────────────────┘
```

### MemTable (記憶體表)

寫入的第一站，使用 `std::collections::BTreeMap<Vec<u8>, Vec<u8>>` 實作。

特性：
- 所有 put/delete 操作先寫入 MemTable
- 內部使用 BTreeMap 保持鍵值有序
- 達到大小閾值後凍結並刷寫為 SSTable

### WAL (Write-Ahead Log)

為保證持久性，每次寫入 MemTable 的同時也寫入 WAL：
- 檔案位置：`<path>/wal.log`
- 崩潰復原：啟動時回放 WAL 中的操作，重建 MemTable
- 成功刷寫後清除 WAL

```rust
impl Wal {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn append(&self, key: &[u8], val: &[u8]) -> Result<()>;
    pub fn recover(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    pub fn clear(&self) -> Result<()>;
}
```

### SSTable (Sorted String Table)

MemTable 刷寫後產生的不可變磁碟檔案：
- 鍵值資料以排序方式儲存
- 支援 Bloom Filter 加速不存在鍵的查詢
- 使用 `serde` + `bincode` 進行序列化

```rust
pub struct SSTable {
    path: PathBuf,
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    bloom: BloomFilter,
}
```

### Bloom Filter

用於快速判斷鍵是否不存在於 SSTable 中：
- 若 Bloom Filter 回傳「不存在」，則跳過該 SSTable 的磁碟查詢
- 若回傳「可能存在」，仍需實際查詢
- 使用 1024 bit 的位元陣列

## 主要操作

### Put (寫入)

1. 寫入 WAL（持久化）
2. 寫入 MemTable（記憶體）
3. 若 MemTable 大小超標，觸發 flush

### Get (讀取)

1. 查詢 MemTable（最新資料）
2. 若未找到，依序查詢各層 SSTable（從最新到最舊）
3. 每層先用 Bloom Filter 過濾
4. 回傳第一個找到的值（墓碑 tombstone 值表示已刪除）

### Delete (刪除)

以 put(key, tombstone) 方式實作（不實際清除舊資料）：
1. 寫入一筆特殊的刪除標記到 MemTable
2. 後續 compaction 時再實體清除

### Flush (刷寫)

將 MemTable 轉換為 SSTable：
1. 凍結目前 MemTable（停止接受寫入）
2. 將 MemTable 中的有序資料序列化為 `.sst` 檔案
3. 同時訓練 Bloom Filter
4. 建立新的空 MemTable
5. 清除 WAL

### Compaction (合併)

週期性合併多個 SSTable 以控制檔案數量並回收空間：
1. 選取需要合併的 SSTable 集合
2. 讀取所有 SSTable 的鍵值，進行多路合併排序
3. 寫入新的 SSTable（跳過已刪除的鍵）
4. 刪除舊的 SSTable

## LSM vs BTree 在 db6 中的比較

本專案的 db6 支援三種可插拔引擎，透過 `StorageEngine` trait 統一介面：

```rust
pub trait StorageEngine: Sized {
    fn open(path: &Path) -> Result<Self>;
    fn open_memory() -> Result<Self>;
    fn get(&self, table_id: u32, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, table_id: u32, key: &[u8], val: &[u8]) -> Result<()>;
    fn delete(&self, table_id: u32, key: &[u8]) -> Result<()>;
    fn scan(&self, table_id: u32, range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    fn flush(&self) -> Result<()>;
}
```

| 特性 | LsmEngine | BTreeEngine | MemoryEngine |
|---|---|---|---|
| 持久性 | 磁碟（SSTable + WAL） | 磁碟 | 無（純記憶體） |
| 寫入效能 | ⭐⭐⭐（循序寫入） | ⭐（隨機寫入） | ⭐⭐⭐ |
| 讀取效能 | ⭐⭐（可能需多層查詢） | ⭐⭐⭐ | ⭐⭐⭐ |
| 空間放大 | 中（需 compaction） | 低 | 無 |
| 寫入放大 | 高（重複 compaction） | 低 | 無 |
| 使用場景 | 寫入密集 | 讀取密集 | 測試/快取 |

## db6 中的使用方式

### 切換引擎

在 REPL 中動態切換：
```
db6> .engine lsm
db6> .engine btree
db6> .engine memory
```

### 程式碼中使用

```rust
use db6::engine::{LsmEngine, StorageEngine};

let engine = LsmEngine::open("/tmp/mydb")?;
engine.put(1, b"key1", b"value1")?;
let val = engine.get(1, b"key1")?;
```

## 效能特性

### 優點
- 寫入吞吐量極高（將隨機 I/O 轉換為循序 I/O）
- 天然的 append-only 寫入模式
- 壓縮率高（資料在 SSTable 中連續存放）

### 缺點
- 讀取可能需查詢多個 SSTable（讀取放大）
- Compaction 消耗 CPU 與 I/O 資源（寫入放大）
- 空間放大（舊版本 SSTable 在 compaction 前佔用空間）
- 不適合點查詢密集的場景（除非使用 Bloom Filter）

## 實作細節

### 執行緒安全

`LsmEngine` 使用 `RwLock` 確保執行緒安全：
- MemTable 與 SSTable 各自獨立鎖定
- 讀取操作可並行（多個讀取鎖）
- 寫入操作需要寫入鎖

### 事務支援

有限的事務支援（單寫入緩衝）：
```
begin transaction → 寫入 tx_buffer → commit(flush to memtable) 或 rollback(discard)
```

### table_id

為了在單一引擎中支援多個邏輯資料表，所有 KV 操作接受 `table_id: u32` 參數。table_id 會作為鍵的前綴編碼在實際儲存的鍵中，實現多表隔離。

## 相關檔案

- `database/lsm/src/lsm/engine.rs` — LSM 引擎核心（517 行）
- `database/lsm/src/lsm/memtable.rs` — MemTable 實作
- `database/lsm/src/lsm/sstable.rs` — SSTable 讀寫
- `database/lsm/src/lsm/bloom.rs` — Bloom Filter
- `database/lsm/src/lsm/wal.rs` — Write-Ahead Log
- `database/lsm/src/engine.rs` — StorageEngine trait 定義
- `database/db6/src/engine/lsm.rs` — db6 中對 LSM 的包裝
- `database/db6/src/engine/memory/` — Memory 引擎（對照用）
- `database/db6/src/engine/btree/` — BTree 引擎（對照用）

## 參考資料

- Patrick O'Neil 等人, "The Log-Structured Merge-Tree (LSM-Tree)", Acta Informatica 1996
- LevelDB：https://github.com/google/leveldb
- RocksDB：https://rocksdb.org/
- LSM-Tree 視覺化解說：https://github.com/ayende/hello-lsm

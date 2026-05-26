# B-Tree

## 概述

B-Tree 是一種自平衡的樹狀資料結構，專為磁碟或區塊儲存裝置設計。與二元搜尋樹不同，B-Tree 的每個節點可以包含多個鍵與多個子節點，透過減少樹的高度來降低磁碟 I/O 次數。B-Tree 廣泛用於資料庫索引與檔案系統中。

本專案包含兩個 B-Tree 相關實作：`database/btree/`（獨立 BTree crate）與 `database/db6/src/engine/btree/`（db6 的 BTree 引擎）。

## B-Tree 的基本性質

### 定義

一棵階數為 m 的 B-Tree 滿足以下性質：

1. 每個節點最多有 m 個子節點
2. 每個非根節點至少有 ⌈m/2⌉ 個子節點
3. 根節點至少有 2 個子節點（除非為葉節點）
4. 有 k 個子節點的節點包含 k-1 個鍵
5. 所有葉節點在同一層級

### 節點結構

```rust
// B-Tree 節點（本專案實作）
struct BTreeNode<K, V> {
    keys: Vec<K>,           // 鍵，已排序
    values: Vec<V>,         // 值（或可選）
    children: Vec<BTreeNode<K, V>>, // 子節點（非葉節點）
    is_leaf: bool,          // 是否為葉節點
    max_keys: usize,        // 節點最大鍵數
}
```

### 範例（階數 5）

```
            [10, 20, 30]
           /    |   \    \
          /     |    \    \
    [5,7]   [15,17]  [25]  [35,40,45]
```

- 每個節點最多有 4 個鍵（m-1）
- 所有葉節點在同一層
- 內部節點的鍵分割子節點的範圍

## 搜尋 (Search)

B-Tree 搜尋類似二元搜尋樹，但在每個節點中進行二元搜尋決定下一步：

```rust
fn search(&self, key: &K) -> Option<&V> {
    let mut current = &self.root;
    loop {
        match current.keys.binary_search(key) {
            Ok(idx) => return Some(&current.values[idx]),
            Err(idx) => {
                if current.is_leaf {
                    return None;
                }
                current = &current.children[idx];
            }
        }
    }
}
```

時間複雜度：O(log_m n)，其中 m 為階數、n 為節點數。

## 插入 (Insert)

插入總是從葉節點開始。若葉節點未滿，直接插入；若已滿，則進行**分裂 (split)**：

```
節點已滿 [3, 5, 7, 9] 插入 6
    │
    ▼
分裂為:
[5]           ← 中位數提升到父節點
 / \
[3] [6, 7, 9]  ← 左右子節點各一半
```

```rust
fn insert(&mut self, key: K, value: V) -> Option<V> {
    let root = &mut self.root;
    if root.keys.len() == root.max_keys {
        // 根節點已滿 → 建立新根
        let mut new_root = BTreeNode::new(false, root.max_keys);
        std::mem::swap(root, &mut new_root);
        let child = Box::new(new_root);
        root.children.push(child);
        root.split_child(0);
    }
    root.insert_non_full(key, value)
}
```

## 刪除 (Delete)

B-Tree 刪除最複雜，需處理三種情況：

1. **鍵在葉節點且節點有足夠鍵** → 直接刪除
2. **鍵在內部節點** → 以左子樹最大值或右子樹最小值取代
3. **鍵所在節點鍵數不足** → 先從兄弟節點借鍵（或合併節點）

## 本專案的 BTree 引擎

`database/btree/` 提供一個持久化的 B-Tree 實作。

### API

```rust
impl BTreeEngine {
    pub fn new() -> Self;
    pub fn open(path: &Path) -> Result<Self>;
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>>;
    pub fn delete(&self, key: &[u8]) -> Option<Vec<u8>>;
    pub fn scan(&self, range: Range<Vec<u8>>) -> Vec<(Vec<u8>, Vec<u8>)>;
    pub fn flush(&self) -> Result<()>;
}
```

### 持久化

- 透過 `serde` + `bincode` 序列化到磁碟
- 支援 WAL 確保當機復原
- 在記憶體中操作，透過 `flush()` 寫回磁碟

### db6 的 BTreeEngine

`database/db6/src/engine/btree/` 將 B-Tree 包裝為 `StorageEngine` trait 的實作：

```rust
pub struct BTreeEngine {
    inner: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
    path: Option<PathBuf>,
    // ...
}

impl StorageEngine for BTreeEngine {
    fn get(&self, table_id: u32, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, table_id: u32, key: &[u8], value: &[u8]) -> Result<()>;
    // ...
}
```

使用 `RwLock` 確保執行緒安全。

## B-Tree 的變體

### B+ Tree

B+ Tree 是 B-Tree 最常見的變體：
- 所有資料僅儲存在葉節點
- 內部節點只儲存鍵作為路由索引
- 葉節點以鏈結串列相連，支援高效範圍掃描

### B* Tree

要求內部節點至少 2/3 滿（而非 1/2），減少分裂次數。

### LSM-Tree vs B-Tree

| 特性 | B-Tree | LSM-Tree |
|---|---|---|
| 讀取效能 | ⭐⭐⭐ 穩定 O(log n) | ⭐⭐ 需多層查詢 |
| 寫入效能 | ⭐ 隨機寫入（直接修改頁面） | ⭐⭐⭐ 循序寫入（append-only） |
| 空間放大 | 低（就地更新） | 中（需 compaction） |
| 寫入放大 | 低 | 高（多次 compaction） |
| 磁碟定址 | 頁面層級（4/8/16KB） | SSTable 層級 |
| 典型用途 | 讀取密集、OLTP | 寫入密集、OLAP |

## 本專案的 BTree vs 標準 BTreeMap

```rust
// Rust 標準函式庫的 BTreeMap
use std::collections::BTreeMap;  // B+ Tree 實作

// 本專案的 BTree 引擎
use btree::BTreeEngine;  // 持久化 KV 儲存
```

| 特性 | std::collections::BTreeMap | 本專案 BTreeEngine |
|---|---|---|
| 持久性 | 無 | 有（序列化到磁碟） |
| 執行緒安全 | 無（需包裝） | 有（RwLock） |
| 值型別 | V（泛型） | Vec<u8> |
| 階數 | 6 (B+) | 可配置 |
| 序列化 | 無 | bincode |
| 範圍掃描 | range() | scan() |

## 相關檔案

- `database/btree/src/lib.rs` — 獨立 BTree crate
- `database/db6/src/engine/btree/` — db6 的 BTree 引擎包裝
- `database/btree/test.sh` — 測試腳本

## 參考資料

- R. Bayer, E. McCreight, "Organization and Maintenance of Large Ordered Indices", 1970
- D. Comer, "The Ubiquitous B-Tree", ACM Computing Surveys, 1979
- *Introduction to Algorithms* (CLRS), Chapter 18: B-Trees

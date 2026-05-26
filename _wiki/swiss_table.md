# Swiss Table

## 概述

Swiss Table 是 Google 開發的高效能雜湊表演算法，最初於 2017 年在 [Abseil](https://abseil.io/) 函式庫中發布（`absl::flat_hash_map`）。其名稱源自 Swiss 團隊（Google 在瑞士蘇黎世的辦公室）。與傳統的開放定址法雜湊表不同，Swiss Table 利用 SIMD 指令進行批次探測，在現代處理器上大幅提升快取效率與查找速度。

本專案的實作位於 `database/swisstable/` crate。

## 核心設計

### 控制位元組 (Control Bytes)

Swiss Table 的核心創新是將每個 bucket 的控制資訊儲存在獨立的控制位元組陣列中：

```
控制位元組陣列: [c0, c1, c2, c3, ..., cN-1]
Bucket 陣列:     [b0, b1, b2, b3, ..., bN-1]
```

每個控制位元組編碼了所對應 bucket 的狀態：

```rust
const TOMBSTONE: u64 = u64::MAX;  // 已刪除標記

pub struct Bucket<K, V> {
    hash: u64,               // 完整 64 位元雜湊值
    key: Option<Box<K>>,     // 鍵
    value: Option<Box<V>>,   // 值
}
```

在 Google 的原始實作中，控制位元組使用雜湊值的高 7 位元加上 1 位元狀態標記（empty/tombstone）。本專案的簡化實作將狀態資訊編碼在 `hash` 欄位中（`u64::MAX` 表示 tombstone，`0` 表示空）。

### 探測策略：Robin Hood Hashing

本實作採用 Robin Hood hashing（羅賓漢雜湊）作為探測策略。當插入一個新鍵值時，若其理想位置已被佔用，則比較現有鍵的「探測距離」（與理想位置的偏移量）。若新鍵已走的距離超過現有鍵，則交換兩者，並繼續為被踢出的鍵尋找位置。這樣做能最小化探測序列長度的變異數，避免長探測鏈。

```rust
pub fn insert(&mut self, mut key: K, mut value: V) -> Option<V> {
    let mut hash = self.hash_key(&key);
    let mut index = self.probe_index(hash);
    let mut dist = 0usize;

    loop {
        let bucket = &mut self.buckets[index];

        if bucket.key.is_none() || bucket.hash == TOMBSTONE {
            // 空位或 tombstone：直接插入
            bucket.hash = hash;
            bucket.key = Some(Box::new(key));
            bucket.value = Some(Box::new(value));
            self.len += 1;
            return None;
        }

        if bucket.key.as_ref().map(|k| **k == key).unwrap_or(false) {
            // 已存在：更新值
            let old_value = bucket.value.take().unwrap();
            bucket.value = Some(Box::new(value));
            return Some(*old_value);
        }

        // Robin Hood：若新鍵的偏移超過現有鍵，則交換
        let existing_index = self.probe_index(bucket.hash);
        let existing_dist = (index - existing_index) & (self.capacity - 1);
        if dist > existing_dist {
            std::mem::swap(&mut bucket.hash, &mut hash);
            std::mem::swap(&mut bucket.key, &mut key);
            std::mem::swap(&mut bucket.value, &mut value);
            dist = existing_dist;
        }

        dist += 1;
        index = (index + 1) & (self.capacity - 1);

        if dist >= self.capacity {
            self.resize();
            return self.insert(key, value);
        }
    }
}
```

### 查詢

查詢操作沿著 probe 序列線性掃描，直到遇到空 bucket（未找到）或找到匹配項：

```rust
pub fn get(&self, key: &K) -> Option<&V> {
    let hash = self.hash_key(key);
    let mut index = self.probe_index(hash);

    loop {
        let bucket = &self.buckets[index];

        if bucket.key.is_none() {
            return None;           // 空 bucket → 停止
        }
        if bucket.hash == TOMBSTONE {
            return None;           // 墓碑 → 停止（簡化版，Google 版會繼續）
        }
        if bucket.hash == hash && bucket.key.as_ref().map(|k| **k == *key).unwrap_or(false) {
            return bucket.value.as_ref().map(|v| &**v);
        }

        index = (index + 1) & (self.capacity - 1);
        if index == self.probe_index(hash) {
            return None;           // 繞了一圈
        }
    }
}
```

### 刪除與墓碑 (Tombstone)

刪除操作使用延遲刪除（lazy deletion）策略：
1. 找到目標 bucket
2. 將 key 與 value 設為 None
3. 將 hash 設為 `TOMBSTONE`（`u64::MAX`）

墓碑標記在插入時可被覆寫，但在查詢時會觸發提前停止（簡化版行為）。

## 本專案的實作細節

### 容量管理

- 容量總是 2 的冪次，最小 16
- 初始容量：`4.next_power_of_two().max(16) = 16`
- 擴展因子：當 probe 距離超過容量時，觸發 2x 擴充

```rust
fn resize(&mut self) {
    let new_cap = self.capacity * 2;
    // 建立新 bucket 陣列
    // 重新雜湊所有舊 bucket
    // 釋放舊陣列
}
```

### 去重支援

- `insert()` 傳回舊值（若鍵已存在）
- `remove()` 傳回被刪除的值

### 雙向迭代器

支援 `iter()` 與 `IntoIterator`，走訪所有非空 bucket：

```rust
pub fn iter(&self) -> Iter<'_, K, V> {
    self.into_iter()
}
```

## 對比 std::collections::HashMap

| 特性 | Rust std HashMap | 本專案 Swiss Table |
|---|---|---|
| 演算法 | 瑞士表格 (since Rust 1.72) | 自製瑞士表格 |
| 探測 | Simd-controlled | Robin Hood |
| 處理衝突 | Swiss Table | Robin Hood |
| 刪除策略 | 墓碑 + 清理 | 墓碑 |
| no_std | 否 | 是（with alloc） |
| API 表面積 | 完整 | 基礎（insert/get/remove/iter/clear） |
| SIMD | 是 | 無（純軟體探測） |

本專案的瑞士表格實作沒有使用 SIMD 指令（不同於 Google 原始版與 Rust std 的 hashbrown），而是以 Robin Hood hashing 作為探測最佳化策略。

## SwissTableSet

同時提供 `SwisstableSet<T>`，以 `SwisstableMap<T, ()>` 為底層：

```rust
pub struct SwisstableSet<T> {
    map: SwisstableMap<T, ()>,
}
```

支援的操作：`insert`、`contains`、`remove`、`len`、`is_empty`、`clear`、`iter`。

## 記憶體佈局

所有 bucket 在單一連續記憶體區塊中配置：

```rust
let layout = Layout::array::<Bucket<K, V>>(capacity).unwrap();
let buckets = unsafe { alloc(layout) as *mut Bucket<K, V> };
```

使用 `unsafe` 進行手動記憶體管理（無 Vec 包裝），適合 no_std 環境。

## 效能

### 優點
- 快取友好（連續記憶體存取）
- 探測距離變異數小（Robin Hood）
- 不需額外記憶體配置直到擴展
- no_std 相容（僅需 alloc）

### 與 hashbrown（Rust std 使用）的差異
- hashbrown 使用控制位元組 + SIMD 批次探測
- 本實作每個 bucket 存放完整 64 位元 hash（hashbrown 僅存 7 位元）
- 本實作的記憶體 overhead 較大（每個 bucket 多存 hash）
- hashbrown 的墓碑在擴展時清理；本實作需手動 resize

## 使用範例

```rust
use swisstable::SwisstableMap;

let mut map = SwisstableMap::new();
map.insert("key1", 100);
map.insert("key2", 200);
assert_eq!(map.get(&"key1"), Some(&100));
assert_eq!(map.remove(&"key2"), Some(200));
assert!(map.is_empty() == false);
```

```rust
use swisstable::SwisstableSet;

let mut set = SwisstableSet::new();
set.insert(1);
set.insert(2);
assert!(set.contains(&1));
```

## 測試覆蓋

包含 25 個測試：
- 新建為空
- 單一/多筆插入
- 更新現有鍵
- 刪除
- 大量插入（100 筆）
- 疊代
- 容量檢查
- Default trait
- Debug 輸出
- 字串鍵
- Set 操作（插入、含重複、刪除、疊代）
- 清空 (clear)
- Index 運算子

## 相關檔案

- `database/swisstable/src/lib.rs` — 完整實作（含 SwisstableMap、SwisstableSet、Iter、803 行）
- `database/swisstable/examples/` — 範例

## 參考資料

- Google Abseil Swiss Table 設計文件：https://abseil.io/docs/cpp/guides/container
- hashbrown (Rust 的 Swiss Table 實作)：https://github.com/rust-lang/hashbrown
- "Swiss Tables and the absl::flat_hash_map"：https://www.youtube.com/watch?v=ncHmEUmJZf4

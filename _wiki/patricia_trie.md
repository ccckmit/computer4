# Patricia Trie

## 概述

Patricia Trie（Practical Algorithm to Retrieve Information Coded in Alphanumeric，實用字母數字資訊檢索演算法），又稱 Radix Tree（基數樹）或壓縮前綴樹 (Compressed Trie)，是 trie（前綴樹）的一種空間最佳化變體。在標準 trie 中，每個字元作為一條邊，導致大量單子節點（僅有一個子節點的節點）浪費記憶體。Patricia Trie 將連續的單子節點壓縮合併為單一節點，大幅減少節點數量。

本專案的實作位於 `database/patricia-trie/` crate，使用 Rust 2024 edition，無外部依賴。

## 核心資料結構

### 節點定義

```rust
pub struct PatriciaTrie<V> {
    root: Node<V>,
    size: usize,
}

struct Node<V> {
    key: String,                              // 節點儲存的鍵段（字串片段）
    value: Option<V>,                          // 可選的值（中間節點可能無值）
    children: BTreeMap<String, Box<Node<V>>>,  // 子節點（以首字元索引）
}
```

每個節點儲存：
- `key`：字串片段（非完整鍵，而是與父節點路徑的差異部分）
- `value`：若此節點對應一個完整的鍵，則儲存值
- `children`：以子節點鍵的第一個字元為索引的 BTreeMap

### 壓縮原理

假設插入 "test"、"testing"、"tester"：

```
標準 trie（未壓縮）:
root → t → e → s → t → i → n → g
                      → e → r

Patricia trie（壓縮後）:
root → "test" → "ing"
              → "er"
```

節點從 10 個減少到 4 個。壓縮率隨著鍵的長度與共享前綴比例增加而提升。

## 核心演算法

### 尋找最長共享前綴 (Longest Common Prefix)

```rust
fn longest_common_prefix_length(key: &str, child_key: &str) -> usize {
    let bytes = key.as_bytes()
        .iter()
        .zip(child_key.as_bytes())
        .take_while(|(x, y)| x == y)
        .count();
    let mut lcp = bytes;
    // 確保停在字元邊界（支援 Unicode）
    while lcp > 0 && !key.is_char_boundary(lcp) {
        lcp -= 1;
    }
    lcp
}
```

Unicode 安全：使用 `is_char_boundary()` 確保切分點落在合法的 UTF-8 字元邊界。

### 插入

插入操作的核心挑戰是維護樹的壓縮性質。當新鍵與現有子節點共享部分前綴時，需要**分裂節點** (node splitting)：

```rust
fn insert_recursive(node: &mut Node<V>, key: &str, value: V) -> Option<V> {
    // 1. 若 key 為空，在此節點儲存值
    if key.is_empty() {
        let old = node.value.take();
        node.value = Some(value);
        return old;
    }

    let fc = first_char(key);

    // 2. 若無對應子節點，直接建立新子節點
    if !node.children.contains_key(&fc) {
        let mut child = Node::new(key);
        child.value = Some(value);
        node.children.insert(fc, Box::new(child));
        return None;
    }

    let child_key = node.children[&fc].key.clone();
    let lcp = longest_common_prefix_length(key, &child_key);

    if lcp == child_key.len() {
        // 3. 完整匹配子節點鍵 → 遞迴插入剩餘部分
        let child = node.children.get_mut(&fc).unwrap();
        return Self::insert_recursive(child, &key[lcp..], value);
    }

    // 4. 部分匹配 → 分裂節點
    let mut child = node.children.remove(&fc).unwrap();

    // 建立剩餘節點（原子節點的剩餘部分）
    let mut rest = Node::new(&child.key[lcp..]);
    rest.value = child.value.take();
    rest.children = std::mem::take(&mut child.children);

    // 更新原子節點為分裂後的前綴節點
    child.key = child.key[..lcp].to_string();
    child.value = None;

    // 將剩餘節點加入前綴節點
    let rest_fc = first_char(&rest.key);
    child.children.insert(rest_fc, Box::new(rest));

    // 遞迴插入新鍵的剩餘部分
    let old = Self::insert_recursive(&mut child, &key[lcp..], value);
    node.children.insert(fc, child);
    old
}
```

### 查詢

查詢沿樹向下走訪，逐段匹配字串：

```rust
fn get_recursive<'a>(node: &'a Node<V>, key: &str) -> Option<&'a V> {
    if key.is_empty() {
        return node.value.as_ref();
    }

    let fc = first_char(key);
    match node.children.get(&fc) {
        None => None,
        Some(child) => {
            if key.starts_with(&child.key) {
                Self::get_recursive(child, &key[child.key.len()..])
            } else {
                None
            }
        }
    }
}
```

### 刪除

刪除包含節點合併 (node merging) 的反向操作：

```rust
fn delete_internal(node: &mut Node<V>, key: &str) -> (Option<V>, bool) {
    // 1. 找到目標鍵
    // 2. 刪除值
    // 3. 若子節點只剩一個且自身無值，與唯一子節點合併
    // 4. 回傳是否應將此節點從父節點移除
}
```

合併條件：
- 節點自身無值
- 僅有一個子節點
- 非根節點

滿足條件時，將子節點的鍵附加到目前節點的鍵上，並接手子節點的子節點與值。

### 前綴搜尋 (Prefix Search)

找出所有以指定前綴開頭的鍵：

```rust
pub fn prefix_search(&self, prefix: &str) -> Vec<(String, &V)> {
    // 1. 沿樹走訪直到完全消耗前綴字串
    // 2. 收集該節點以下的所有鍵值對
}
```

### 最長前綴匹配 (Longest Prefix Match)

找出最長的前綴鍵（常用於 IP 路由表）：

```rust
pub fn longest_prefix(&self, key: &str) -> Option<(String, &V)> {
    // 沿樹走訪，記錄最後一個有值的節點
    // 回傳最長匹配
}
```

## 本專案實作的特點

### Unicode 支援

- 使用 `first_char()` 以第一個字元（非第一個位元組）作為子節點索引
- UTF-8 安全的字串切割
- 中文、表情符號等多位元組字元皆可正確處理

```rust
fn first_char(s: &str) -> String {
    s.chars().next().unwrap().to_string()
}
```

### BTreeMap 子節點

使用 `BTreeMap<String, Box<Node<V>>>` 而非 HashMap 作為子節點容器：
- 子節點保持排序（方便調試與疊代）
- BTreeMap 的範圍查詢未來可擴展
- 子節點數量通常很小（26 個字母 + 數字），BTreeMap 的 log(n) 效能足夠

### 16 個核心 API

| 方法 | 說明 |
|---|---|
| `new()` | 建立空樹 |
| `insert(key, value)` | 插入或更新鍵值 |
| `get(key)` | 查詢鍵對應的值 |
| `contains(key)` | 檢查鍵是否存在 |
| `delete(key)` | 刪除鍵值 |
| `keys()` | 回傳所有鍵 |
| `values()` | 回傳所有值的參照 |
| `iter()` | 回傳所有鍵值對 |
| `prefix_search(prefix)` | 前綴搜尋 |
| `longest_prefix(key)` | 最長前綴匹配 |
| `len()` | 樹的大小 |
| `is_empty()` | 是否為空 |
| `clone()` | 深拷貝 |
| `clear()` | 清空（需要實作） |
| `Default` trait | 空建構子 |
| `Debug` trait | 調試輸出 |

## 效能分析

### 時間複雜度

| 操作 | 平均 | 最差 |
|---|---|---|
| `insert` | O(k) | O(k) |
| `get` | O(k) | O(k) |
| `delete` | O(k) | O(k) |
| `prefix_search` | O(k + m) | O(n) |
| `longest_prefix` | O(k) | O(k) |

k = 鍵長度，m = 匹配結果數量，n = 節點數

### 空間複雜度

- 優於標準 trie（壓縮共享前綴）
- 節點數最多為鍵數的 2 倍（每個分裂操作增加一個節點）
- 每字串平均儲存 overhead 約 2 個位元組（取決於共享前綴）

### 與 HashMap 的比較

| 特性 | Patricia Trie | HashMap |
|---|---|---|
| 有序性 | 前綴有序（keys() 排序回傳） | 無序 |
| 前綴搜尋 | O(k + m) | 不支援（需掃描全部） |
| 最長前綴 | O(k) | 不支援 |
| 空間效率 | 共享前綴節省空間 | 每鍵獨立儲存 |
| 雜湊碰撞 | 無 | 需處理碰撞 |
| Unicode 鍵 | 最適（共享位元組前綴） | 位元組雜湊 |

## 測試覆蓋

包含 35 個測試：
- 空樹操作
- 基本插入與查詢
- 更新現有鍵
- 空字串鍵
- 前綴擴展（test → testing → tester）
- 節點分裂 (cat/car/bat)
- 葉節點刪除
- 節點合併刪除
- 串聯合併刪除
- 內部節點刪除
- 空鍵刪除
- 前綴搜尋（含無匹配、空前綴）
- 最長前綴匹配（含根節點值）
- keys/values/iter
- 大量插入（1000 筆）
- Unicode（中文、café）
- Clone
- 插入再全部刪除

## 相關檔案

- `database/patricia-trie/src/lib.rs` — 完整實作（646 行含測試）

## 參考資料

- Morrison, D. R. "PATRICIA — Practical Algorithm to Retrieve Information Coded in Alphanumeric", Journal of the ACM, 1968
- Radix tree (Wikipedia)：https://en.wikipedia.org/wiki/Radix_tree
- 與 Rust 標準 `collections` 的比較：HashMap 無前綴搜尋功能

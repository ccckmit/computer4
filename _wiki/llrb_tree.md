# LLRB 樹 (Left-Leaning Red-Black Tree)

## 概述

LLRB（Left-Leaning Red-Black Tree，左傾紅黑樹）是紅黑樹的一種變體，由 Robert Sedgewick 於 2008 年提出。與傳統的紅黑樹相比，LLRB 強化了兩條額外限制：紅色節點只能為左子節點（left-leaning），且禁止連續兩個紅色左節點。這些限制大幅簡化了插入與刪除的實作，將紅黑樹的 6 種旋轉情況減少到 3 種。

本專案實作於 `database/redblacktree/` crate，使用 Rust 2024 edition，無外部依賴。

## 節點定義

```rust
enum Color {
    Red,
    Black,
}

struct Node<K, V> {
    key: K,
    value: V,
    color: Color,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
}

pub struct RedBlackTree<K, V> {
    root: Option<Box<Node<K, V>>>,
    size: usize,
}
```

泛型限制：`K: Ord + Clone`、`V: Clone`

## LLRB 的紅黑樹規則

標準紅黑樹：
1. 每個節點是紅色或黑色
2. 根節點是黑色
3. 葉節點 (None) 是黑色
4. 紅色節點的子節點必須是黑色（不允許連續紅色）
5. 從任一節點到其葉子節點的所有路徑包含相同數量的黑色節點

LLRB 額外限制：
1. **紅色節點必須為左子節點**（無紅色右子節點）
2. **禁止連續紅色左節點**（無紅-紅左鏈）

這兩條限制保證樹的平衡性，同時將插入修復歸納為 3 種局部操作。

## 核心操作

### 顏色輔助函式

```rust
fn is_red(node: &Option<Box<Node<K, V>>>) -> bool {
    node.as_ref().map_or(false, |n| n.color == Color::Red)
}
```

### 旋轉

#### 左旋轉

當出現紅色右子節點時，將其旋轉至左側：

```rust
fn rotate_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
    let mut right = node.right.take().unwrap();
    node.right = right.left.take();
    let old_color = node.color;
    node.color = Color::Red;        // 節點變紅
    right.color = old_color;         // 右子繼承原顏色
    right.left = Some(node);
    right
}
```

```
旋轉前:        旋轉後:
    Bk              Rr
   /  \            /  \
 ...   Rr         Bk  ...
       / \       / \
     ...  ...   ... ...
```

#### 右旋轉

當出現連續紅色左節點時：

```rust
fn rotate_right(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
    let mut left = node.left.take().unwrap();
    node.left = left.right.take();
    let old_color = node.color;
    node.color = Color::Red;
    left.color = old_color;
    left.right = Some(node);
    left
}
```

### 顏色翻轉

當左右子節點皆為紅色時，將它們和父節點的顏色交換：

```rust
fn flip_colors(node: &mut Box<Node<K, V>>) {
    node.color = match node.color {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    };
    if let Some(ref mut l) = node.left {
        l.color = match l.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
    }
    if let Some(ref mut r) = node.right {
        r.color = match r.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
    }
}
```

### 插入修復

插入新節點（總是紅色）後，透過 `fix_insert` 恢復 LLRB 性質：

```rust
fn fix_insert(node: Box<Node<K, V>>) -> Box<Node<K, V>> {
    let mut node = node;

    // 情況 1：紅色右子節點 → 左旋轉
    if Self::is_red(&node.right) && !Self::is_red(&node.left) {
        node = Self::rotate_left(node);
    }

    // 情況 2：連續紅色左節點 → 右旋轉
    if Self::is_red(&node.left)
        && node.left.as_ref().map_or(false, |l| Self::is_red(&l.left))
    {
        node = Self::rotate_right(node);
    }

    // 情況 3：左右皆紅 → 顏色翻轉
    if Self::is_red(&node.left) && Self::is_red(&node.right) {
        Self::flip_colors(&mut node);
    }

    node
}
```

這三種情況的處理順序是固定的，且涵蓋了所有不平衡的可能性。

### 插入流程

```rust
pub fn insert(&mut self, key: K, value: V) {
    if !self.contains(&key) {
        self.size += 1;
    }
    let new_node = Box::new(Node {
        key, value,
        color: Color::Red,     // 新節點總是紅色
        left: None, right: None,
    });
    self.root = Self::insert_recursive(self.root.take(), new_node);
    // 根節點總是黑色
    if let Some(ref mut root) = self.root {
        root.color = Color::Black;
    }
}

fn insert_recursive(node: Option<Box<Node<K, V>>>, new_node: Box<Node<K, V>>)
    -> Option<Box<Node<K, V>>>
{
    match node {
        None => Some(new_node),
        Some(mut n) => {
            match new_node.key.cmp(&n.key) {
                Ordering::Equal => { n.value = new_node.value; Some(n) }
                Ordering::Less => {
                    n.left = Self::insert_recursive(n.left.take(), new_node);
                    Some(Self::fix_insert(n))
                }
                Ordering::Greater => {
                    n.right = Self::insert_recursive(n.right.take(), new_node);
                    Some(Self::fix_insert(n))
                }
            }
        }
    }
}
```

### 刪除

刪除是 LLRB 中最複雜的操作。核心策略是確保在向下走訪時，目前節點不會是 2-node（黑色且無紅色子節點），方法是必要時從兄弟節點借一個鍵來使其成為 3-node。

```rust
fn move_red_left(mut node: Box<Node<K, V>>) -> Box<Node<K, V>> {
    // 將紅色從右子節點移動到左子節點
    Self::flip_colors(&mut node);
    if let Some(ref mut right) = node.right {
        if Self::is_red(&right.left) {
            node.right = Some(Self::rotate_right(right.clone()));
            node = Self::rotate_left(node);
            Self::flip_colors(&mut node);
        }
    }
    node
}
```

刪除演算法：
1. 在樹中向下走訪目標鍵
2. 沿途保證節點為 3-node（非 2-node）
3. 找到目標後，以右子樹最小值取代（或左子樹最大值）
4. 向上回溯時修復 LLRB 性質

## 平衡保證

LLRB 保證從根到葉子的所有路徑上的黑色節點數相同（黑色高度平衡），最長路徑長度不超過最短路徑的 2 倍：

- 一個節點到葉子的最短路徑：全黑節點
- 最長路徑：黑紅交錯（紅色節點只能為左子，禁止連續紅）

因此最長路徑長度 ≤ 2 × 最短路徑長度，保證樹高 O(log n)。

## 本專案的實作特點

### API 設計

```rust
// 基本操作
pub fn new() -> Self
pub fn size(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn contains(&self, key: &K) -> bool
pub fn get(&self, key: &K) -> Option<&V>
pub fn insert(&mut self, key: K, value: V)
pub fn delete(&self, key: &K) -> bool

// 遍歷
pub fn min_key(&self) -> Option<K>
pub fn max_key(&self) -> Option<K>
pub fn keys(&self) -> Vec<&K>
pub fn values(&self) -> Vec<&V>
pub fn iter(&self) -> Vec<(&K, &V)>
pub fn clear(&mut self)

// 範圍操作
pub fn range(&self, low: &K, high: &K) -> Vec<(&K, &V)>
pub fn floor(&self, key: &K) -> Option<&K>
pub fn ceiling(&self, key: &K) -> Option<&K>

// 順序統計
pub fn select(&self, k: usize) -> Option<&K>
pub fn rank(&self, key: &K) -> usize
```

### 遞迴實作

本實作使用遞迴而非迭代：
- 插入與刪除使用遞迴後，在回溯時進行旋轉與顏色修復
- Rust 的遞迴深度限制（通常 512）足夠處理合理大小的樹（O(log n)）
- 程式碼較迭代版本更簡潔

### 不可變查詢

`get`、`contains`、`min_key`、`max_key` 使用迭代走訪（而非遞迴），避免堆疊開銷：

```rust
pub fn get(&self, key: &K) -> Option<&V> {
    let mut current = &self.root;
    while let Some(node) = current {
        match key.cmp(&node.key) {
            Ordering::Equal => return Some(&node.value),
            Ordering::Less => current = &node.left,
            Ordering::Greater => current = &node.right,
        }
    }
    None
}
```

### CLI 問題

注意 `src/main.rs` 的 CLI 是無狀態的：每個命令建立一個全新的空樹。這在 `database/redblacktree/AGENTS.md` 中有說明，是設計限制而非錯誤。

## 測試覆蓋

包含 26 個行內測試：
- 基本操作（new、size、is_empty）
- 插入與查詢
- 更新值
- 刪除（葉節點、內部節點、根節點）
- 重複刪除
- 遍歷（min、max、keys、values、iter）
- 範圍操作（range、floor、ceiling）
- 順序統計（select、rank）
- 大量插入與刪除
- 疊代器走訪
- 與 BTreeMap 行為比較

## 效能比較

| 操作 | LLRB 平均 | BTree (B=6) |
|---|---|---|
| 插入 | O(log n) | O(log n) |
| 查詢 | O(log n) | O(log n) |
| 刪除 | O(log n) | O(log n) |
| 範圍掃描 | O(k + log n) | O(k + log n) |
| 空間 | ~3 指標/節點 | ~B 鍵/節點 |
| 快取效能 | 較差（指標追蹤） | 較好（連續陣列） |

LLRB 對比 BTree：LLRB 適合需要有序性的場景且實作較簡單；BTree 則在大量資料時具備更好的快取區域性。

## 相關檔案

- `database/redblacktree/src/lib.rs` — LLRB 完整實作（910 行含測試）
- `database/redblacktree/src/main.rs` — 無狀態 CLI
- `database/redblacktree/examples/` — 4 個範例 (basic, iterator_demo, stress_test, string_keys)
- `database/redblacktree/AGENTS.md` — 套件專屬指令

## 參考資料

- Robert Sedgewick, "Left-leaning Red-Black Trees", 2008：https://sedgewick.io/talks/llrb.pdf
- Robert Sedgewick and Kevin Wayne, *Algorithms* 4th Edition, Chapter 3.3
- Wikipedia：https://en.wikipedia.org/wiki/Red%E2%80%93black_tree
- LLVM 的 BTree 實作對照：`database/btree/`

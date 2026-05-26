# 資料庫 (Database)

## 概述

資料庫是結構化資料的集合，提供高效率的儲存、查詢與管理功能。本專案的旗艦資料庫 `db6` 是一個多模型資料庫，整合了 KV、SQL、全文檢索 (FTS) 與訊息佇列 (Msgq) 等功能，並支援可插拔的儲存引擎（Memory、BTree、LSM）。

## 資料庫模型

### 鍵值 (KV) 資料庫

最簡單的資料庫模型，以鍵值對的方式儲存資料：

```
key1 → value1
key2 → value2
...
```

- **優點：** 高效能、可水平擴展
- **典型用途：** 快取、工作階段管理
- **本專案：** db6 的 KV API 是所有引擎的基礎介面

### 關聯式資料庫 (RDBMS)

資料以表格 (table) 形式儲存，表格由行 (row) 與列 (column) 組成，支援 SQL 查詢：

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT UNIQUE
);

SELECT * FROM users WHERE name LIKE '張%';
```

- **優點：** ACID 事務、豐富查詢語法、資料完整性
- **本專案：** db6 的 SQL 層（parser → planner → executor）

### 文件資料庫 (Document DB)

儲存半結構化資料（JSON/BSON）：

```json
{
    "_id": "user123",
    "name": "張三",
    "email": "zhang@example.com",
    "tags": ["developer", "rust"]
}
```

### 圖形資料庫 (Graph DB)

以節點與邊的形式儲存關係資料，適合社交網路、推薦系統。

## db6 的架構

```
┌──────────────────────────────────────────────┐
│  使用者介面層                                  │
│  REPL (互動式) │  Client/Server │  gRPC API   │
├──────────────────────────────────────────────┤
│  查詢層                                       │
│  SQL Parser → Planner → Executor              │
├──────────────────────────────────────────────┤
│  索引層                                       │
│  FTS (全文檢索) │  BTree 索引 │  自訂索引      │
├─────────────────┬────────────────┬───────────┤
│  儲存引擎層      │                │           │
│  MemoryEngine  │  BTreeEngine  │  LsmEngine │
│  (BTreeMap)    │  (RwLock)     │  (LSM-Tree)│
├─────────────────┴────────────────┴───────────┤
│  KV API 抽象層 (StorageEngine trait)          │
│  get / put / delete / scan                    │
├──────────────────────────────────────────────┤
│  底層                                         │
│  WAL │ 序列化 (bincode) │ 協定緩衝 (protobuf) │
└──────────────────────────────────────────────┘
```

### 儲存引擎特徵 (StorageEngine trait)

```rust
pub trait StorageEngine: Sized {
    fn open(path: &Path) -> Result<Self>;
    fn open_memory() -> Result<Self>;
    fn get(&self, table_id: u32, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, table_id: u32, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&self, table_id: u32, key: &[u8]) -> Result<()>;
    fn scan(&self, table_id: u32,
            range: (Bound<Vec<u8>>, Bound<Vec<u8>>)) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;
    fn flush(&self) -> Result<()>;
}
```

## 資料庫的關鍵技術

### ACID 事務

| 特性 | 說明 | 本專案支援 |
|---|---|---|
| A (Atomicity) | 事務內操作全部成功或全部失敗 | LSM 引擎支援 |
| C (Consistency) | 事務前後資料保持一致 | 有限支援 |
| I (Isolation) | 並行事務互不干擾 | 行列層級 |
| D (Durability) | 已提交事務永久保存 | WAL 確保 |

### 索引 (Index)

加速查詢的資料結構：

| 索引類型 | 適用查詢 | 底層結構 |
|---|---|---|
| 主鍵索引 | 點查詢 | KV (BTree/LSM) |
| 次要索引 | 條件查詢 | BTree |
| 全文索引 | 文字搜尋 | 反向索引 (FTS) |

### 查詢最佳化

SQL 查詢的執行計劃：

```sql
SELECT u.name, o.total
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE u.age > 18
ORDER BY o.total DESC
LIMIT 10;
```

最佳化器選擇的執行計劃：
```
1. 在 users 上掃描 age > 18
2. 對每個符合條件的 user，在 orders 上索引查找
3. 合併結果
4. 依 total 排序
5. 取前 10 筆
```

### 快取 (Caching)

減少磁碟 I/O：
- Buffer pool：快取常用資料頁
- WAL buffer：批次寫入日誌
- Bloom filter：快速判斷鍵是否存在

## db6 的 REPL 使用

```bash
$ cd database/db6
$ cargo run

db6> .engine lsm
db6> CREATE TABLE users (id INT, name TEXT, email TEXT);
db6> INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
db6> SELECT * FROM users WHERE name = 'Alice';
db6> .engine memory              # 切換引擎
db6> SELECT * FROM users;        # 新引擎中資料為空
db6> .quit
```

## 與其他資料庫的比較

| 特性 | db6 | SQLite | Redis | PostgreSQL |
|---|---|---|---|---|
| 模型 | KV + SQL + FTS | 關聯式 | KV | 關聯式 |
| 儲存引擎 | 可插拔 (Mem/BTree/LSM) | BTree | 記憶體 + RDB | Heap/BTree |
| 多執行緒 | RwLock | 執行緒安全 | 單執行緒 | 行程/執行緒 |
| 網路協定 | gRPC + HTTP | 無 (嵌入) | RESP | libpq |
| 全文檢索 | CJK Bigram FTS | FTS5 | RediSearch | GIN |
| 訊息佇列 | 內建 msgq | 無 | Pub/Sub | LISTEN/NOTIFY |
| 語言 | Rust | C | C | C |

## 相關檔案

- `database/db6/src/lib.rs` — db6 入口與公開 API
- `database/db6/src/engine/` — 儲存引擎實作
- `database/db6/src/sql/` — SQL 解析器、規劃器、執行器
- `database/db6/src/fts/` — 全文檢索
- `database/db6/src/msgq/` — 訊息佇列
- `database/db6/src/server/` — gRPC/HTTP 伺服器
- `database/db6/AGENTS.md` — 開發者注意事項

## 參考資料

- C. J. Date, *An Introduction to Database Systems*
- SQLite 架構：https://www.sqlite.org/arch.html
- db6 規劃文件：`database/db6/_doc/plan.md`

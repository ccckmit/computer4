# SQL

## 概述

SQL (Structured Query Language) 是操作關聯式資料庫的標準語言，涵蓋資料定義 (DDL)、資料操作 (DML)、資料查詢 (DQL) 與資料控制 (DCL)。本專案的 `db6` 支援 SQL 查詢，包含自製的 SQL 解析器 (parser)、規劃器 (planner) 與執行器 (executor)。

## SQL 語句分類

### DDL (Data Definition Language)
```sql
CREATE TABLE users (id INTEGER, name TEXT, email TEXT);
CREATE INDEX idx_name ON users(name);
DROP TABLE users;
ALTER TABLE users ADD COLUMN age INTEGER;
```

### DML (Data Manipulation Language)
```sql
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
UPDATE users SET email = 'alice@new.com' WHERE id = 1;
DELETE FROM users WHERE id = 1;
```

### DQL (Data Query Language)
```sql
SELECT id, name FROM users WHERE age > 18 ORDER BY name LIMIT 10;
SELECT COUNT(*), AVG(age) FROM users GROUP BY city;
SELECT u.name, o.total FROM users u JOIN orders o ON u.id = o.user_id;
```

### DCL (Data Control Language)
```sql
GRANT SELECT ON users TO 'readonly';
REVOKE INSERT ON users FROM 'readonly';
```

## SQL 查詢執行流程

```
SELECT u.name, o.total
FROM users u JOIN orders o ON u.id = o.user_id
WHERE u.age > 18 AND o.total > 100
ORDER BY o.total DESC
LIMIT 10;

    │
    ▼
┌─────────────────────┐
│  1. Parser (解析器)  │
│  SQL 文字 → AST      │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│  2. Planner (規劃器) │
│  AST → 執行計劃      │
│  (選擇索引、JOIN 順序)│
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│  3. Executor (執行器)│
│  執行計劃 → 結果集    │
└─────────────────────┘
```

## db6 的 SQL 實作

### SQL 解析器

`database/db6/src/sql/parser/` 實作 SQL 語法解析：

```rust
// db6 SQL parser 入口
pub fn parse(sql: &str) -> Result<Statement>;

// 支援的 SQL 語法
pub enum Statement {
    CreateTable(CreateTable),
    Insert(Insert),
    Select(Select),
    Update(Update),
    Delete(Delete),
    DropTable(DropTable),
    CreateIndex(CreateIndex),
}
```

### SQL 規劃器

`database/db6/src/sql/planner/` 將 AST 轉換為執行計劃：

```rust
pub struct Plan {
    pub operator: PlanNode,
}

pub enum PlanNode {
    SeqScan { table: String, filter: Option<Expr> },
    IndexScan { table: String, index: String, key: Expr },
    NestedLoopJoin { left: Box<PlanNode>, right: Box<PlanNode>, cond: Expr },
    Filter { input: Box<PlanNode>, predicate: Expr },
    Projection { input: Box<PlanNode>, columns: Vec<String> },
    Sort { input: Box<PlanNode>, key: String, desc: bool },
    Limit { input: Box<PlanNode>, count: usize },
    Aggregation { input: Box<PlanNode>, agg: AggFunc, group_by: Vec<String> },
}
```

### SQL 執行器

`database/db6/src/sql/executor/` 執行計劃並回傳結果：

```rust
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}
```

## 支援的 SQL 語法

```sql
-- 資料表操作
CREATE TABLE users (id INTEGER, name TEXT, email TEXT, age INTEGER);
DROP TABLE users;

-- 資料插入
INSERT INTO users VALUES (1, 'Alice', 'alice@test.com', 30);
INSERT INTO users (id, name) VALUES (2, 'Bob');

-- 查詢
SELECT * FROM users;
SELECT id, name FROM users WHERE age > 18;
SELECT * FROM users ORDER BY name;
SELECT * FROM users ORDER BY age DESC LIMIT 5;
SELECT COUNT(*), AVG(age) FROM users;
SELECT city, COUNT(*) FROM users GROUP BY city;

-- JOIN
SELECT u.name, o.total
FROM users u
JOIN orders o ON u.id = o.user_id;

-- 全文檢索（db6 擴充）
SELECT * FROM docs WHERE body MATCH '人工智慧';
SELECT * FROM docs WHERE body MATCH '人工智慧 AND 機器學習';
SELECT * FROM docs WHERE body MATCH '人工智慧 OR 機器學習';

-- 更新與刪除
UPDATE users SET email = 'new@test.com' WHERE id = 1;
DELETE FROM users WHERE id = 1;
```

## 支援的 SQL 功能比較

| 功能 | db6 | SQLite |
|---|---|---|
| CREATE TABLE | ✓ | ✓ |
| INSERT | ✓ | ✓ |
| SELECT + WHERE | ✓ | ✓ |
| ORDER BY | ✓ | ✓ |
| LIMIT | ✓ | ✓ |
| GROUP BY / 聚合 | ✓ | ✓ |
| JOIN | ✓ (Nested Loop) | ✓ (多種) |
| 子查詢 | 有限 | ✓ |
| 全文檢索 (FTS) | MATCH 子句 | FTS5 |
| 事務 | BEGIN/COMMIT | ✓ |
| 索引 | CREATE INDEX | ✓ |
| 外部鍵 | 有限 | ✓ |
| UNION | 無 | ✓ |
| WINDOW | 無 | ✓ |
| CTE | 無 | ✓ |

## db6 REPL 中使用 SQL

```sh
$ cd database/db6
$ cargo run

db6> .engine btree
db6> CREATE TABLE products (id INTEGER, name TEXT, price REAL);
db6> INSERT INTO products VALUES (1, '蘋果', 30.0);
db6> INSERT INTO products VALUES (2, '香蕉', 15.0);
db6> SELECT * FROM products WHERE price > 20;
 id │ name │ price
────┼──────┼───────
 1  │ 蘋果 │ 30.0
────┴──────┴───────
db6> .engine lsm
db6> SELECT * FROM products;
(empty — 新引擎無資料)
```

## SQL 執行在 KV 引擎之上

db6 的 SQL 層建構在 KV API 之上：

```
SQL 表格 → KV 命名空間規則

CREATE TABLE users (...)
  → KV table_id = 1

INSERT INTO users VALUES (1, 'Alice')
  → engine.put(1, pk_encoding, row_encoding)

SELECT * FROM users WHERE id = 1
  → engine.get(1, pk_encoding)
  → decode row

SELECT * FROM users WHERE age > 18
  → engine.scan(1, full_range)
  → 對每行檢查 age > 18
```

## 相關檔案

- `database/db6/src/sql/parser/` — SQL 解析器
- `database/db6/src/sql/planner/` — 查詢規劃器
- `database/db6/src/sql/executor/` — 查詢執行器
- `database/db6/src/sql/mod.rs` — SQL 模組入口
- `database/db6/src/sql/sql.md` — SQL 實作文件
- `database/sql4/` — 獨立 SQLite-like crate

## 參考資料

- SQL 標準：ISO/IEC 9075
- SQLite 文件：https://www.sqlite.org/docs.html
- db6 SQL 實作：`database/db6/src/sql/sql.md`

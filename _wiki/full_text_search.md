# 全文檢索 (Full-Text Search, FTS)

## 概述

全文檢索 (FTS) 是一種讓使用者以關鍵字或詞組搜尋文件內容的技術，與 SQL 的 `LIKE '%keyword%'` 不同，FTS 透過預先建立反向索引 (Inverted Index) 來加速搜尋，支援更複雜的查詢語意如 AND/OR、相關性排序、斷詞處理等。

本專案的 FTS 實作位於 `database/fts/` crate，並整合至 `database/db6/` 旗艦資料庫中。

## 核心資料結構：反向索引 (Inverted Index)

反向索引是 FTS 的核心，將每個詞項對應到包含該詞項的文件清單：

```
詞項 "人工" → [doc0, doc2, doc3]
詞項 "智慧" → [doc0, doc1, doc2]
詞項 "機器" → [doc0, doc3]
```

本專案的實作：

```rust
pub struct Index {
    inverted: HashMap<String, Vec<usize>>,  // 詞項 → 文件 ID 清單
    docs: Vec<String>,                       // 文件內容（以 ID 索引）
}
```

### 反向索引的建立

1. 輸入一行文字作為一份文件
2. 對文字進行斷詞 (tokenize)
3. 對每個詞項進行去重（同一文件中同一詞項僅記錄一次）
4. 將詞項加入反向索引，對應到該文件 ID

```rust
pub fn add_doc(&mut self, line: &str) {
    let doc_id = self.docs.len();
    self.docs.push(line.to_string());
    let terms = tokenize(line);
    let mut seen = std::collections::HashSet::new();
    for term in terms {
        if seen.insert(term.clone()) {
            self.inverted.entry(term).or_default().push(doc_id);
        }
    }
}
```

## CJK 斷詞器

### 中文斷詞的挑戰

不同於英文以空格分隔單詞，中文（以及其他 CJK 語言）的文字之間沒有明確的分隔符號。簡單的字元比對無法處理複合詞。常見策略：

1. **單字分詞 (Unigram)：** 每個漢字為一個詞（精準度低）
2. **二元分詞 (Bigram)：** 每兩個連續漢字為一個詞（平衡效率與品質）
3. **詞典分詞：** 使用預先建立的詞典進行最大匹配（準確度高但需要詞典維護）
4. **統計分詞：** 使用機器學習模型（準確度最高但成本也最高）

### 本專案採用：Bigram + ASCII 保留

本專案的實作採用二元分詞法 (Bigram)，同時保留 ASCII 詞彙：

```rust
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut run_is_ascii = false;

    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '+' || ch == '-' {
            if !run_is_ascii && !buf.is_empty() {
                tokens.extend(cjk_bigrams(&buf));  // 先前 CJK 部分切為 bigram
                buf.clear();
            }
            run_is_ascii = true;
            buf.push(ch);
        } else if is_cjk(ch) {
            if run_is_ascii && !buf.is_empty() {
                tokens.push(buf.clone().to_lowercase());  // 先前 ASCII 部分保留
                buf.clear();
            }
            run_is_ascii = false;
            buf.push(ch);
        } else {
            // 標點符號：觸發邊界切分
            if !buf.is_empty() {
                if run_is_ascii {
                    tokens.push(buf.clone().to_lowercase());
                } else {
                    tokens.extend(cjk_bigrams(&buf));
                }
                buf.clear();
            }
        }
    }
    // ... flush remaining buffer
}
```

對於輸入「人工智慧，機器學習！」，產生的詞項為：
```
"人工" "工智" "智慧" "機器" "器學" "學習"
```

### Bigram 實作

```rust
fn cjk_bigrams(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();
    if chars.len() == 1 {
        result.push(chars[0].to_string());  // 單字直接返回
    } else {
        for i in 0..chars.len() - 1 {
            let bigram: String = chars[i..=i + 1].iter().collect();
            result.push(bigram);
        }
    }
    result
}
```

### CJK 字元範圍

```rust
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |   // CJK 統一表意文字
        '\u{3400}'..='\u{4DBF}' |   // CJK 擴展 A
        '\u{F900}'..='\u{FAFF}' |   // CJK 相容表意文字
        '\u{2F800}'..='\u{2FA1F}'   // CJK 擴展 B
    )
}
```

### 斷詞範例

| 輸入 | 輸出詞項 |
|---|---|
| `"hello world"` | `["hello", "world"]` |
| `"人工智慧"` | `["人工", "工智", "智慧"]` |
| `"人"` | `["人"]` |
| `"AI 人工智慧"` | `["ai", "人工", "工智", "智慧"]` |
| `"Hello World"` | `["hello", "world"]`（小寫化） |
| `"人工智慧，機器學習！"` | `["人工", "工智", "智慧", "機器", "器學", "學習"]` |
| `"state-of-the-art"` | `["state-of-the-art"]`（連字號保留） |
| `"CJK 2.0 FTS"` | `["cjk", "2.0", "fts"]` |

## 搜尋模式

支援兩種搜尋模式：

### OR 模式

查詢詞項中任一項匹配即算命中，結果依相關性分數排序：

```rust
SearchMode::Or
```

相關性分數 = 匹配詞項數 / 查詢詞項總數

例如查詢「人工智慧 機器學習」：
- 文件「機器學習是人工智慧的重要分支」→ 分數 2/2 = 1.0
- 文件「人工智慧」→ 分數 1/2 = 0.5

### AND 模式

所有查詢詞項都必須出現才算命中：

```rust
SearchMode::And
```

例如查詢「人工智慧 機器學習」：
- 僅包含兩詞的文件才回傳
- 結果按分數排序（分數計算同 OR）

## 在 db6 中的整合

`database/db6/` 將 FTS 整合為 SQL 語法中的全文搜尋功能：

### SQL 語法

```sql
-- 建立 FTS 索引（自動）
CREATE VIRTUAL TABLE docs USING fts5(title, body);

-- 全文搜尋
SELECT * FROM docs WHERE body MATCH '人工智慧';

-- 布林搜尋
SELECT * FROM docs WHERE body MATCH '人工智慧 AND 機器學習';
SELECT * FROM docs WHERE body MATCH '人工智慧 OR 機器學習';
```

### db6 中的 FTS 實作

`database/db6/src/fts/` 目錄包含：
- `CjkTokenizer` — 與 `database/fts/` `tokenize()` 相容的中文斷詞器
- `EnglishTokenizer` — 英文斷詞器（空白 + 標點分割）
- `FtsIndex` — FTS 索引管理（建立、查詢、維護）

### FTS 與 KV 引擎的互動

db6 將 FTS 索引儲存在底層 KV 引擎之上：
- 反向索引作為 KV pair 儲存（詞項 → posting list）
- 文件內容另存於 files table
- 支援事務性更新（寫入 KV 引擎時同時更新 FTS 索引）

## 使用建議

### 適用場景
- 中英文混合的文件搜尋
- 知識庫、文件管理系統、筆記應用
- 需要比 SQL LIKE 更高效率的文字搜尋

### 限制與注意事項
- Bigram 分詞可能產生「無意義」的詞項（如「人工」與「工智」同時出現）
- 不支援詞幹提取 (stemming) 與同義詞擴展
- 混淆字（如簡體/繁體轉換）需自行預處理
- 大規模文件需定期重建索引以維持效能

## 與其他技術的比較

| 特性 | 本專案 FTS | SQLite FTS5 | Elasticsearch |
|---|---|---|---|
| CJK 支援 | Bigram | 需外掛 ICU tokenizer | ICU 分析器 |
| 索引結構 | HashMap 反向索引 | B-Tree 分段索引 | Lucene 倒排索引 |
| 分詞演算法 | Bigram | 可自訂 | ICU/自訂分析器 |
| 相關性排序 | 簡單分數（詞項匹配比） | BM25 | TF-IDF/BM25 |
| 分散式支援 | 無 | 無 | 原生分散式 |
| 語言 | Rust | C | Java |

## 相關檔案

- `database/fts/src/lib.rs` — FTS 核心實作（tokenize、Index、search，379 行含測試）
- `database/db6/src/fts/` — db6 中的 FTS 整合層
  - `cjk_tokenizer.rs` — CJK 斷詞器
  - `english_tokenizer.rs` — 英文斷詞器
  - `fts_index.rs` — 索引管理
- `database/db6/src/sql/` — SQL 解析器，包含 MATCH 子句處理

## 參考資料

- SQLite FTS5 Extension：https://www.sqlite.org/fts5.html
- 資訊檢索導論 (M. Manning 等人)：https://nlp.stanford.edu/IR-book/
- CJK 分詞技術：https://en.wikipedia.org/wiki/CJK_information_processing

# 瀏覽器 (Browser)

## 概述

網頁瀏覽器是使用者存取全球資訊網 (WWW) 的核心軟體，負責載入、解析與渲染 HTML、CSS、JavaScript 內容。本專案包含三個瀏覽器實作：`browser4`（使用 eframe + boa_engine）、`browser5`（使用 eframe + 自製 xdom4/js4）、`md4browser`（專注於 Markdown 渲染）。

## 瀏覽器的核心元件

```
URL 輸入
    │
    ▼
HTTP 請求 (載入 HTML)
    │
    ▼
HTML 解析 (DOM 建立)
    │
    ├── CSS 解析 (樣式計算)
    │
    ├── JavaScript 執行 (DOM 操作)
    │
    ▼
佈局計算 (Layout)
    │
    ▼
繪製 (Painting/Rendering)
    │
    ▼
畫面輸出
```

## browser4

`web/browser4/` 使用成熟的第三方函式庫快速實作。

### 技術堆疊

| 功能 | 使用函式庫 |
|---|---|
| GUI | eframe / egui |
| HTML 解析 | scraper (HTML5) |
| CSS | 基礎 inline CSS |
| JavaScript | boa_engine (完整 JS 引擎) |
| 網路 | reqwest |
| 中文字型 | 內嵌 font.ttf |

### 核心結構

```rust
struct Browser4 {
    url_input: String,
    dom: Option<Html>,           // scraper 的 HTML 文件
    js_context: Context,         // boa_engine JS 上下文
    page_content: String,        // 純文字頁面內容
    is_loading: bool,
    history: Vec<String>,
}
```

### 支援的功能

- 輸入 URL 載入頁面
- HTML 解析與 DOM 建立
- 基本 CSS 樣式支援（顏色、字型、大小）
- 完整 JavaScript 執行（透過 boa_engine）
- 瀏覽歷史（上一頁/下一頁）
- 中文字型渲染

## browser5

`web/browser5/` 是一個更深入的自製瀏覽器，使用自訂的 xdom4（DOM）與 js4（JS 引擎）。

### 架構

```
┌─────────────────────────────────────┐
│  eframe / egui (GUI 框架)            │
├─────────────────────────────────────┤
│  html.rs (HTML 解析器)               │
│  css.rs (CSS 解析與選擇器)            │
│  renderer.rs (佈局與渲染)             │
│  js.rs (JavaScript 橋接)             │
├─────────────────────────────────────┤
│  xdom4 (自訂 DOM 實作)               │ ← 區域依賴
│  js4 (自訂 JS 引擎)                  │ ← 區域依賴
└─────────────────────────────────────┘
```

### 自訂 DOM (xdom4)

`web/xdom4/` 實作 XML/DOM 函式庫：

```rust
// xdom4 的節點型別
pub enum NodeType {
    Element(ElementNode),
    Text(String),
    Document,
    Comment(String),
}

pub struct ElementNode {
    pub tag: String,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Node>,
}
```

支援 CSS 選擇器：標籤名、ID、class、屬性、子代、後代選擇器。

### 自訂 JS 引擎 (js4)

`web/js4/` 實作 JavaScript 引擎，包含完整管線：

```
JavaScript 原始碼
    │
    ▼
Tokenizer (詞法分析)
    │
    ▼
Parser (Pratt parsing → AST)
    │
    ▼
Interpreter (樹走訪直譯)
    │
    ▼
執行結果
```

支援的特性：
- let、const 變數
- 函式定義與呼叫（含閉包）
- if/else、while 控制流
- try/catch 例外處理
- 陣列與物件字面值
- console.log 輸出
- DOM API: `document.getElementById()`, `element.innerText`

### browser5 的渲染流程

```rust
fn load_url(&mut self, url: String) {
    // 1. 載入 HTML
    let html = fetch(url);
    // 2. 解析 DOM
    let dom = html::parse(&html);
    // 3. 提取 CSS
    let css_rules = css::extract(&dom);
    // 4. 提取 JavaScript
    let scripts = html::extract_scripts(&dom);
    // 5. 執行 JS（操作 DOM）
    for script in scripts {
        self.js_runtime.execute(&script, &mut dom);
    }
    // 6. 渲染
    renderer.render(&dom, &css_rules, ui);
}
```

### 區域路徑依賴

不同於其他 crate 的獨立性，browser5 依賴本專案內的其他 crate：

```toml
# web/browser5/Cargo.toml
[dependencies]
xdom4 = { path = "../xdom4" }
js4 = { path = "../js4" }
```

這意味著：這些 crate 必須一起建置、測試順序需注意。

## md4browser

`web/md4browser/` 是 Markdown 專用的簡化瀏覽器：

```sh
cd web/md4browser
cargo run https://example.com/readme.md
cargo run /path/to/local/file.md
```

使用 egui_commonmark 進行 Markdown 渲染 + reqwest 載入遠端內容。

## 瀏覽器比較

| 特性 | browser4 | browser5 | md4browser |
|---|---|---|---|
| JS 引擎 | boa_engine（完整） | js4（自製，精簡） | 無 |
| DOM 引擎 | scraper（HTML5） | xdom4（自製） | 無 |
| CSS | 基礎 inline | xdom4 CSS 選擇器 | 無 |
| 格式 | HTML | HTML | Markdown |
| 依賴 | 外部 crate | 自製 + 區域路徑 | egui_commonmark |
| 網路 | reqwest | 檔案/HTTP | reqwest |
| 字型 | 內嵌 font.ttf | 系統字型 | 內嵌 font.ttf |

## 相關檔案

- `web/browser4/src/main.rs` — browser4 主程式（522 行）
- `web/browser5/src/main.rs` — browser5 主程式（278 行）
- `web/browser5/src/html.rs` — HTML 解析器
- `web/browser5/src/css.rs` — CSS 解析器
- `web/browser5/src/renderer.rs` — 佈局與渲染器
- `web/browser5/src/js.rs` — JS 橋接
- `web/xdom4/src/` — DOM 函式庫
- `web/js4/src/` — JavaScript 引擎
- `web/md4browser/src/main.rs` — Markdown 瀏覽器

## 參考資料

- HTML 標準：https://html.spec.whatwg.org/
- CSS 標準：https://www.w3.org/Style/CSS/
- ECMAScript 規格 (JavaScript)：https://tc39.es/ecma262/
- eframe/egui 文件：https://docs.rs/egui/

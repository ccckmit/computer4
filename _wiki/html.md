# HTML

## 概述

HTML (HyperText Markup Language) 是全球資訊網的核心標記語言，由 WHATWG 維護的 living standard。HTML 文件由元素 (element) 組成的 DOM 樹描述網頁結構。本專案的 `web/browser4/` 使用 `scraper` crate 解析 HTML，`web/browser5/` 使用自訂解析器 + `xdom4` DOM 函式庫。

## HTML 文件結構

```html
<!DOCTYPE html>
<html lang="zh-TW">
<head>
    <meta charset="UTF-8">
    <title>頁面標題</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <h1>主標題</h1>
    <p>段落文字 <a href="https://example.com">連結</a></p>
    <script src="app.js"></script>
</body>
</html>
```

### 元素結構

```
開始標籤 (opening tag): <tagname attr="value">
內容 (content): 文字或子元素
結束標籤 (closing tag): </tagname>

<p class="intro">Hello World</p>
├─開始標籤───┘├─內容──┤├─結束標籤┤
```

## 本專案的 HTML 解析

### browser4（使用 scraper）

```rust
use scraper::{Html, Selector};

fn parse_html(html_str: &str) {
    let doc = Html::parse_document(html_str);
    let selector = Selector::parse("p.intro").unwrap();
    for element in doc.select(&selector) {
        println!("{}", element.text().collect::<String>());
    }
}
```

### browser5（自製解析器）

`web/browser5/src/html.rs` 實作從零開始的 HTML 解析器：

```rust
// 簡化版解析流程
pub fn parse(input: &str) -> Document {
    let tokens = tokenize(input);   // 詞法分析
    let dom = build_dom(tokens);    // DOM 樹建構
    dom
}

// Token 型別
enum Token {
    OpenTag { name: String, attrs: Vec<(String, String)> },
    CloseTag(String),
    Text(String),
    Comment(String),
    Doctype(String),
    SelfClosingTag { name: String, attrs: Vec<(String, String)> },
}
```

### 與 xdom4 的整合

browser5 將解析後的 DOM 轉換為 xdom4 的節點結構：

```rust
use xdom4::{Node, Element};

fn scraper_to_xdom4(scraper_node: &scraper::ElementRef) -> Node {
    let tag = scraper_node.value().name().to_string();
    let attrs: HashMap<String, String> = scraper_node
        .value()
        .attrs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Node::Element(Element { tag, attrs, children: vec![] })
}
```

## HTML 語法寬容

HTML 解析器須處理許多不合規範的輸入：

| 特性 | 規範 | 本專案 |
|---|---|---|
| 未閉合標籤 | 不允許 | 自動推斷閉合 |
| 標籤名大小寫 | 大小寫不拘 | 轉小寫 |
| 屬性值<br>引號省略 | 可省略（特定情況） | 支援 |
| 自訂標籤 | 允許 | 支援 |
| 註解 `<!-- -->` | 支援 | 支援 |
| `<!DOCTYPE>` | 可忽略 | 支援 |

## 常見 HTML 元素分類

| 類別 | 元素範例 |
|---|---|
| 區段 | `<header>`, `<nav>`, `<main>`, `<section>`, `<footer>` |
| 文字 | `<h1>`–`<h6>`, `<p>`, `<span>`, `<strong>`, `<em>` |
| 串聯 | `<ul>`/`<ol>`/`<li>`, `<dl>`/`<dt>`/`<dd>` |
| 表格 | `<table>`, `<tr>`, `<td>`, `<th>` |
| 表單 | `<form>`, `<input>`, `<button>`, `<select>`, `<textarea>` |
| 媒體 | `<img>`, `<video>`, `<audio>`, `<canvas>` |
| 內嵌 | `<iframe>`, `<embed>`, `<object>` |
| 腳本 | `<script>`, `<noscript>` |
| 中繼 | `<meta>`, `<title>`, `<link>`, `<style>` |

## 相關檔案

- `web/browser5/src/html.rs` — 自製 HTML 解析器
- `web/xdom4/src/` — DOM 函式庫
- `web/browser4/src/main.rs` — scraper 解析 HTML
- `web/js4/src/` — JS 引擎（操作 DOM）

## 參考資料

- HTML Living Standard：https://html.spec.whatwg.org/
- 瀏覽器引擎原理：https://www.html5rocks.com/en/tutorials/internals/howbrowserswork/

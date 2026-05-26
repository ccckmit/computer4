# DOM (Document Object Model)

## 概述

DOM (文件物件模型) 是 HTML 與 XML 文件的程式化表示，提供一個樹狀結構讓程式語言（通常是 JavaScript）動態存取與操作文件內容、結構與樣式。本專案的 `web/xdom4/` crate 實作自製的 DOM 函式庫，用於 `web/browser5/` 瀏覽器。

## DOM 樹結構

```
Document
  │
  ├── <html>
  │     ├── <head>
  │     │     ├── <title>  "頁面標題"
  │     │     └── <meta charset="UTF-8">
  │     │
  │     └── <body>
  │           ├── <h1 id="title">  "歡迎"
  │           └── <p class="intro">  "這是段落"
  │                 └── <a href="...">  "連結"
  │
  └── <!-- 註解 -->
```

### 節點型別

```rust
// xdom4 的節點型別定義
pub enum NodeType {
    Document,
    Element(Element),
    Text(String),
    Comment(String),
    DocumentFragment,
}

pub struct Node {
    pub node_type: NodeType,
    pub parent: Option<Rc<RefCell<Node>>>,
    pub children: Vec<Rc<RefCell<Node>>>,
}
```

## Node 介面

規範定義的 `Node` 介面屬性与方法：

| 屬性/方法 | 說明 | xdom4 狀態 |
|---|---|---|
| `nodeType` | 節點型別常數 | ✓ |
| `nodeName` | 節點名稱 | ✓ |
| `nodeValue` | 節點值（Text 節點） | ✓ |
| `parentNode` | 父節點 | ✓ |
| `childNodes` | 子節點 NodeList | ✓ |
| `firstChild` | 第一個子節點 | ✓ |
| `lastChild` | 最後一個子節點 | ✓ |
| `nextSibling` | 下一個兄弟節點 | ✓ |
| `previousSibling` | 上一個兄弟節點 | ✓ |
| `appendChild(node)` | 新增子節點 | ✓ |
| `removeChild(node)` | 移除子節點 | ✓ |
| `insertBefore(new, ref)` | 插入參考節點前 | ✓ |
| `cloneNode(deep)` | 複製節點 | ✓ |
| `textContent` | 所有子文字內容 | ✓ |

## Element 介面

`Element` 繼承 `Node`，增加元素特有的 API：

```rust
pub struct Element {
    pub tag: String,                       // 標籤名 (小寫)
    pub attrs: HashMap<String, String>,    // 屬性
    pub children: Vec<Node>,
    pub styles: HashMap<String, String>,   // 計算後樣式
    pub id: Option<String>,
    pub classes: Vec<String>,
}
```

| 屬性/方法 | 說明 | xdom4 狀態 |
|---|---|---|
| `tagName` | 標籤名 | ✓ |
| `id` | ID 屬性 | ✓ |
| `className` | class 字串 | ✓ |
| `classList` | class 列表 | ✓ |
| `getAttribute(name)` | 取得屬性值 | ✓ |
| `setAttribute(name, val)` | 設定屬性 | ✓ |
| `hasAttribute(name)` | 是否有屬性 | ✓ |
| `removeAttribute(name)` | 移除屬性 | ✓ |
| `innerHTML` | 內部 HTML（字串） | ✓ |
| `outerHTML` | 外部 HTML | ✓ |
| `innerText` | 呈現文字 | ✓ |
| `textContent` | 所有文字 | ✓ |
| `style` | inline 樣式 | ✓ |
| `children` | 子 Element 集合 | ✓ |
| `querySelector(css)` | CSS 選擇器（單一） | ✓ |
| `querySelectorAll(css)` | CSS 選擇器（全部） | ✓ |
| `getElementsByTagName(name)` | 依標籤名搜尋 | ✓ |
| `getElementsByClassName(name)` | 依 class 搜尋 | ✓ |

## Document 介面

`Document` 是 DOM 樹的根：

```rust
pub struct Document {
    pub doctype: Option<String>,
    pub document_element: Option<Rc<RefCell<Element>>>,
}
```

| 屬性/方法 | 說明 | xdom4 狀態 |
|---|---|---|
| `documentElement` | `<html>` 元素 | ✓ |
| `head` | `<head>` 元素 | ✓ |
| `body` | `<body>` 元素 | ✓ |
| `title` | 頁面標題 | ✓ |
| `getElementById(id)` | 依 ID 找元素 | ✓ |
| `getElementsByTagName(name)` | 依標籤找元素 | ✓ |
| `getElementsByClassName(name)` | 依 class 找元素 | ✓ |
| `createElement(tag)` | 建立元素節點 | ✓ |
| `createTextNode(text)` | 建立文字節點 | ✓ |
| `querySelector(css)` | CSS 選擇器 | ✓ |
| `querySelectorAll(css)` | CSS 選擇器（全部） | ✓ |
| `cookie` | Cookie 字串 | 未實作 |

## DOM 操作範例（browser5 支援）

```javascript
// 查詢元素
let header = document.getElementById("header");
let items = document.getElementsByClassName("item");
let firstBtn = document.querySelector("button.primary");

// 讀寫內容
header.innerText = "新的標題";
let text = header.innerText;

// 操作屬性
header.setAttribute("data-id", "123");
let id = header.getAttribute("data-id");

// 建立與插入
let newDiv = document.createElement("div");
newDiv.innerText = "Hello";
document.body.appendChild(newDiv);

// 樣式操作
header.style.color = "red";
header.style.fontSize = "20px";
```

## CSS 選擇器引擎

xdom4 的 CSS 選擇器支援：

```rust
// 選擇器結構
pub enum Combinator {
    Descendant,  // 空白
    Child,       // >
    Adjacent,    // +
    Sibling,     // ~
}

pub struct Selector {
    pub tag: Option<String>,    // div, p, *
    pub id: Option<String>,     // #myId
    pub classes: Vec<String>,   // .foo.bar
    pub attrs: Vec<AttributeSelector>,
    pub pseudo: Option<PseudoClass>,
}
```

## 瀏覽器整合

browser5 中 DOM、HTML、CSS、JS 的關係：

```
HTML 解析器
    │ 建立
    ▼
DOM 樹 (xdom4)
    │
    ├── CSS 解析器 → 計算樣式 → 渲染樹
    │
    └── JS 引擎 (js4) → 執行程式碼操作 DOM
```

```rust
// browser5 中載入頁面的完整流程
fn load_page(&mut self, url: &str) {
    // 1. 載入原始 HTML
    let html = fetch(url);

    // 2. 解析 HTML 建立 DOM
    let doc = html_parser::parse(&html);

    // 3. 提取 CSS
    let css_rules = css_parser::extract_rules(&doc);

    // 4. 提取並執行 JS
    for script in extract_scripts(&doc) {
        self.js_runtime.execute(&script, &mut doc);
    }

    // 5. 計算樣式
    apply_styles(&mut doc, &css_rules);

    // 6. 渲染
    renderer.render(&doc);
}
```

## 事件模型

DOM 事件在元素樹中傳播：

```
捕獲階段 (Capture)
    Window → Document → <html> → <body> → <div>
                                               │
目標階段 (Target)                              │
    <div> 事件觸發                              │
                                               │
冒泡階段 (Bubble)                              │
    <div> → <body> → <html> → Document → Window
```

xdom4 尚未實作完整事件模型，但瀏覽器環境可透過 JS 引擎模擬基本事件處理。

## 相關檔案

- `web/xdom4/src/lib.rs` — DOM 核心（Node、Element、Document）
- `web/xdom4/src/selector.rs` — CSS 選擇器引擎
- `web/browser5/src/renderer.rs` — 渲染 DOM
- `web/browser5/src/js.rs` — JS 與 DOM 橋接
- `web/browser5/src/html.rs` — HTML → DOM 解析

## 參考資料

- DOM 標準：https://dom.spec.whatwg.org/
- W3C DOM4：https://www.w3.org/TR/dom/
- MDN DOM 文件：https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model
- CSS 選擇器標準：https://www.w3.org/TR/selectors-4/

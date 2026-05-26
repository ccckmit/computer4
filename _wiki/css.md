# CSS

## 概述

CSS (Cascading Style Sheets) 是一種樣式表語言，用於描述 HTML 或 XML 文件的外觀呈現。CSS 將內容（HTML）與呈現（樣式）分離。本專案的 `web/xdom4/` 支援 CSS 選擇器，`web/browser5/` 有基礎的 CSS 解析器與渲染器。

## CSS 語法

```css
/* 選擇器 { 宣告區塊 } */
h1 {
    color: blue;
    font-size: 24px;
}

/* 類別選擇器 */
.intro {
    background-color: #f0f0f0;
    padding: 10px;
}

/* ID 選擇器 + 多屬性 */
#main-title {
    color: #333;
    font-family: "Helvetica Neue", sans-serif;
    font-weight: bold;
    text-align: center;
}
```

### 選擇器的種類

| 選擇器 | 語法 | 範例 |
|---|---|---|
| 標籤 (type) | `tagname` | `p`, `h1`, `div` |
| 類別 (class) | `.classname` | `.intro`, `.highlight` |
| ID | `#idname` | `#header`, `#footer` |
| 屬性 | `[attr]`, `[attr=val]` | `[type="text"]`, `[hidden]` |
| 後代 | `A B` | `div p` |
| 子代 | `A > B` | `ul > li` |
| 相鄰兄弟 | `A + B` | `h1 + p` |
| 偽類別 | `:pseudo` | `:hover`, `:first-child` |

### 層疊與優先級 (Specificity)

```
內嵌樣式 (inline)   >   ID 選擇器   >   類別/屬性/偽類   >   標籤/偽元素
      1000                   100                   10                    1
```

計算方式：

```css
/* specificity = 0-1-0 */
.class-name { color: red; }

/* specificity = 0-0-1 */
p { color: blue; }

/* specificity = 0-1-1 */
p.class-name { color: green; }

/* 同優先級時最後宣告者勝出 */
```

### 繼承與層疊

```css
body { font-family: sans-serif; }      /* 子元素繼承 */
body { margin: 0; }                     /* 子元素不繼承 */
h1 { font-family: serif; }             /* 覆寫繼承 */
```

屬性會繼承：font-family、color、line-height、text-align
屬性不繼承：margin、padding、border、width、height、background

## 本專案的 CSS 實作

### CSS 選擇器（xdom4）

`web/xdom4/src/` 實作 CSS 選擇器引擎：

```rust
// xdom4 CSS 選擇器支援
pub fn query_selector<'a>(root: &'a Node, selector: &Selector) -> Vec<&'a Node>;
pub fn query_selector_all<'a>(root: &'a Node, selector: &Selector) -> Vec<&'a Node>;
```

支�的選擇器：

```rust
pub enum Selector {
    Tag(String),                    // div, p, h1
    Class(String),                  // .className
    Id(String),                     // #idName
    Attribute { name: String, op: AttributeOp, val: Option<String> },
    Descendant(Box<Selector>, Box<Selector>),  // div p
    Child(Box<Selector>, Box<Selector>),        // div > p
    Adjacent(Box<Selector>, Box<Selector>),     // h1 + p
}
```

### CSS 解析器（browser5）

`web/browser5/src/css.rs` 解析 CSS 規則：

```rust
pub struct CssRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

pub struct Declaration {
    pub property: String,
    pub value: String,
}

pub fn parse_css(input: &str) -> Vec<CssRule>;
```

### 樣式計算

```rust
fn apply_styles(node: &mut Node, rules: &[CssRule]) {
    let mut styles = HashMap::new();
    for rule in rules {
        if match_selector(node, &rule.selectors) {
            for decl in &rule.declarations {
                styles.insert(decl.property.clone(), decl.value.clone());
            }
        }
    }
    node.styles = styles;
}
```

### 渲染佈局

browser5 的 `renderer.rs` 使用計算後的樣式進行佈局：

```rust
struct LayoutBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    node: Option<Rc<Node>>,
    children: Vec<LayoutBox>,
}

fn layout(node: &Node, viewport: Size) -> LayoutBox {
    // display: block → 垂直排列
    // display: inline → 水平排列
    // 計算 box model (margin, border, padding, content)
}
```

### 支援的 CSS 屬性

| 屬性 | 狀態 |
|---|---|
| `color` | ✓ 文字顏色 |
| `font-size` | ✓ 字型大小 |
| `font-family` | ✓ 字型名稱 |
| `font-weight` | ✓ 粗體 |
| `background-color` | ✓ 背景色 |
| `margin`, `padding` | ✓ box model |
| `width`, `height` | ✓ 尺寸 |
| `display` | ✓ block / inline / none |
| `text-align` | ✓ 對齊 |
| `border` | 部分支援 |

## 相關檔案

- `web/xdom4/src/lib.rs` — CSS 選擇器實作
- `web/browser5/src/css.rs` — CSS 解析器
- `web/browser5/src/renderer.rs` — 佈局與渲染
- `web/browser4/src/main.rs` — 基礎 inline CSS 支援

## 參考資料

- CSS 標準：https://www.w3.org/Style/CSS/
- CSS 選擇器 Level 4：https://www.w3.org/TR/selectors-4/
- 瀏覽器渲染原理：https://developer.mozilla.org/en-US/docs/Web/Performance/How_browsers_work

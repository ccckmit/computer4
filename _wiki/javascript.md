# JavaScript

## 概述

JavaScript (JS) 是網頁瀏覽器中執行的動態程式語言，也是 ECMAScript 標準的實作。本專案包含兩個 JS 引擎：`web/js4/`（自製精簡引擎）與 `web/browser4/` 使用的 `boa_engine`（完整引擎）。

## ECMAScript 標準

JavaScript 的語言核心由 ECMAScript (ES) 標準定義，每個版本新增特性：

| 版本 | 發布年 | 重要特性 |
|---|---|---|
| ES3 | 1999 | try/catch、正規表達式 |
| ES5 | 2009 | strict mode、JSON、陣列方法 (map/filter/reduce) |
| ES2015 (ES6) | 2015 | let/const、箭頭函式、class、Promise、模組 |
| ES2016+ | 逐年 | async/await、Optional chaining、Nullish coalescing |

## js4 自製引擎

`web/js4/` 實作從零開始的 JavaScript 引擎：

### 管線

```
JavaScript 原始碼
    │
    ▼
Tokenizer (詞法分析)
    │  原始碼 → Token 序列
    ▼
Parser (語法分析)
    │  Token 序列 → AST (抽象語法樹)
    ▼
Interpreter (直譯器)
    │  樹走訪求值
    ▼
執行結果
```

### Tokenizer

```rust
pub enum TokenType {
    // 關鍵字
    Let, Const, Function, Return, If, Else, While, For,
    Try, Catch, Throw, New, This, Class, Var,

    // 運算子
    Plus, Minus, Star, Slash, Eq, EqEq, NotEq,
    Lt, Gt, LtEq, GtEq, AndAnd, OrOr, Not,
    Assign, PlusAssign, MinusAssign,

    // 字面值
    Number(f64), String(String), Bool(bool), Null, Undefined,
    Identifier(String),

    // 符號
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot, Arrow,

    EOF,
}

pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}
```

### AST (抽象語法樹)

```rust
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Identifier(String),
    Binary(Box<Expr>, Op, Box<Expr>),
    Unary(Op, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, Box<Expr>),        // obj.prop
    Index(Box<Expr>, Box<Expr>),         // arr[idx]
    Function(Vec<String>, Vec<Stmt>),    // 函式表示式
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Arrow(Vec<String>, Box<Expr>),
}

pub enum Stmt {
    Expr(Expr),
    Let(String, Option<Expr>),
    Const(String, Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
    Return(Option<Expr>),
    Function(String, Vec<String>, Vec<Stmt>),
    Try(Box<Stmt>, String, Box<Stmt>),
    Throw(Expr),
}
```

### Interpreter (直譯器)

```rust
pub struct Interpreter {
    pub env: Environment,      // 變數環境（支援巢狀作用域）
    pub output: Vec<String>,   // console.log 輸出
}

struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

// 支援的值型別
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Undefined,
    Function { params: Vec<String>, body: Vec<Stmt>, closure: Environment },
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
```

### 作用域與閉包

```rust
fn eval_function_decl(&mut self, params: &[String], body: &[Stmt]) -> Value {
    // 建立閉包：複製目前環境
    let closure = self.env.clone();
    Value::Function {
        params: params.to_vec(),
        body: body.to_vec(),
        closure,
    }
}

// 呼叫函式時建立新環境，以閉包為父環境
fn call_function(&mut self, func: &Value, args: &[Value]) -> Result<Value> {
    match func {
        Value::Function { params, body, closure } => {
            let mut new_env = Environment::new(Some(Box::new(closure.clone())));
            for (param, arg) in params.iter().zip(args.iter()) {
                new_env.define(param, arg.clone());
            }
            let old_env = std::mem::replace(&mut self.env, new_env);
            let result = self.exec_block(body);
            self.env = old_env;  // 恢復環境
            result
        }
        _ => Err("Not a function".into()),
    }
}
```

## 支援的 JS 語法

js4 支援的語法：

```javascript
// 變數
let x = 10;
const y = "hello";
var z = true;

// 運算
let sum = x + 20;
let isBig = x > 5 && y !== null;

// 控制流
if (x > 0) {
    console.log("正數");
} else {
    console.log("非正數");
}

while (x > 0) {
    x--;
}

// 函式
function add(a, b) {
    return a + b;
}
console.log(add(3, 4));

// 閉包
function makeCounter() {
    let count = 0;
    return function() {
        count++;
        return count;
    };
}

// 陣列
let arr = [1, 2, 3];
console.log(arr[0]);

// 物件
let obj = { name: "Alice", age: 30 };
console.log(obj.name);

// 例外
try {
    throw "錯誤";
} catch (e) {
    console.log(e);
}
```

尚未支援的語法：

```javascript
class Foo {}             // class 語法
for (let x of arr) {}     // for...of generator
async/await               // 非同步
Promise                   // Promise
import/export              // ES 模組
Symbol, Map, Set          // 內建物件
```

## browser4 的 boa_engine

`web/browser4/` 使用 `boa_engine`（完整 ECMAScript 引擎）：

```rust
use boa_engine::{Context, Source};

fn execute_js(js_code: &str) {
    let mut context = Context::default();
    match context.eval(Source::from_bytes(js_code)) {
        Ok(result) => println!("Result: {}", result.to_string(&mut context).unwrap()),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

boa_engine 支援完整的 ES2020+ 語法，包括 class、async/await、Promise、模組等。

## DOM API（js4 擴充）

js4 在瀏覽器環境中提供 DOM API：

```rust
// browser5/js.rs — JS 與 DOM 的橋接
impl JsRuntime {
    fn setup_dom_api(&mut self) {
        // document.getElementById(id)
        let get_element_by_id = native_function(|args, runtime| {
            let id = args[0].as_string()?;
            let element = runtime.dom.borrow().get_element_by_id(&id);
            // 回傳 xdom4 的節點參考（以 JS 物件表示）
        });
        runtime.env.define("document", Value::Object({
            let mut doc = HashMap::new();
            doc.insert("getElementById".into(), get_element_by_id);
            doc
        }));
    }
}
```

## 相關檔案

- `web/js4/src/lib.rs` — js4 JS 引擎入口
- `web/js4/src/tokenizer.rs` — 詞法分析器
- `web/js4/src/parser.rs` — Pratt parser
- `web/js4/src/interpreter.rs` — 樹走訪直譯器
- `web/js4/src/environment.rs` — 變數環境與閉包
- `web/browser5/src/js.rs` — 瀏覽器 JS 橋接

## 參考資料

- ECMAScript 規格：https://tc39.es/ecma262/
- boa_engine 文件：https://docs.rs/boa_engine/
- JavaScript 引擎原理：https://developer.mozilla.org/en-US/docs/Mozilla/Projects/SpiderMonkey/Internals

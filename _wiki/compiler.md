# 編譯器 (Compiler)

## 概述

編譯器是將高階程式語言轉換為低階表示式（機器碼、中間碼等）的程式。典型的編譯器由數個階段 (phase) 組成，形成一條編譯管線 (compilation pipeline)。本專案包含一條完整的自製編譯管線：`rustc4`（Rust → LLVM IR）→ `lli4`（LLVM IR 直譯器）。

## 編譯器的階段

### 1. 詞法分析 (Lexical Analysis / Scanning)

將原始碼字串轉換為 token 串流。詞法分析器 (lexer) 讀取字元序列，以正規表達式比對出關鍵字、識別字、運算子、字面值等，忽略空白與註解。

```rust
// rustc4 的 token 枚舉（簡化）
enum Token {
    Ident(String),      // 識別字
    IntLit(i64),        // 整數字面值
    StrLit(String),     // 字串字面值
    Plus, Minus, Star,  // 運算子
    If, Else, While,    // 關鍵字
    Fn, Let, Return,    // 宣告
    LParen, RParen,     // 分隔符
    Semi, Colon,        // 語句終止符
}
```

### 2. 語法分析 (Syntax Analysis / Parsing)

將 token 串流轉換為抽象語法樹 (AST, Abstract Syntax Tree)。語法分析器根據語言的文法規則，決定 token 的結構化組合方式。

常用演算法：
- **遞迴下降解析 (Recursive Descent)：** 為每個文法規則撰寫一個遞迴函式，直觀且易於除錯
- **LR 解析 (LR Parsing)：** 使用堆疊自動機，適合工具生成（如 yacc、bison）

本專案使用遞迴下降解析器。

### 3. 語意分析 (Semantic Analysis)

檢查 AST 是否符合語言的語意規則：
- 型別檢查 (type checking)：運算元型別是否相容
- 變數綁定 (binding)：變數是否已宣告
- 範圍解析 (scope resolution)

### 4. 中間碼生成 (Intermediate Code Generation)

將 AST 轉換為與機器無關的中間表示式 (IR)。本專案使用 LLVM IR 的子集。

### 5. 最佳化 (Optimization)

對 IR 進行轉換以提升執行效率，例如：
- 常數折疊 (constant folding)：`2 + 3` → `5`
- 死碼消除 (dead code elimination)
- 迴圈不變式外提 (loop invariant hoisting)

本專案的編譯器現階段無最佳化。

### 6. 目標碼生成 (Code Generation)

將 IR 轉換為目標機器的指令。傳統編譯器在此階段產生組合語言或機器碼。本專案選擇將 IR 直接交由 lli4 直譯執行，跳過此階段。

## 編譯器 vs 直譯器

| 特性 | 編譯器 (Compiler) | 直譯器 (Interpreter) |
|---|---|---|
| 輸出 | 機器碼 / IR | 無（直接執行） |
| 執行速度 | 快（預先編譯） | 慢（逐句翻譯） |
| 啟動時間 | 慢（需編譯） | 快（立即執行） |
| 除錯體驗 | 較差（需除錯符號） | 較好（行號對應） |
| 動態特性 | 較難支援 | 自然支援 |
| 可攜性 | 需多後端 | 一次實作到處執行 |

## rustc4：本專案的自製編譯器

`compiler/rustc4/` 是 Rust 語言的子集編譯器，輸出 LLVM IR 文字格式。

### 支援的 Rust 子集

```
// 變數宣告與賦值
let x: i32 = 42;
x = x + 1;

// 算術運算
let a = x + y * 2;

// 條件分支
if a > 10 {
    return 1;
} else {
    return 2;
}

// 迴圈
let i = 0;
while i < 10 {
    i = i + 1;
}

// 函式定義與呼叫
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// 陣列操作
let arr: [i32; 5];
arr[0] = 42;
```

### 不支援
- 結構體與列舉
- 特徵 (trait) 與泛型
- 所有權與借用檢查
- 模式匹配 (match)
- 閉包 (closure)
- 巨集 (macro)
- 標準函式庫

### 管線流程

```
Rust 原始碼
    ↓  lexer::tokenize()
Token 串流
    ↓  parser::parse()
AST (抽象語法樹)
    ↓  codegen::generate()
LLVM IR (文字格式)
```

### 公開 API

```rust
// rustc4 的單一入口
pub fn compile(source: &str) -> String;
```

輸入 Rust 原始碼字串，回傳 LLVM IR 文字格式。

## 編譯器相關的理論

### SSA (Static Single Assignment)

LLVM IR 採用 SSA 形式：每個變數僅被賦值一次。這簡化了資料流分析與最佳化：
- `%1 = add i32 %a, %b`
- `%2 = mul i32 %1, %c`

SSA 需要 phi 節點來合併來自不同控制流路徑的值：
```llvm
br i1 %cond, label %then, label %else

then:
  %x1 = add i32 %a, 1
  br label %merge

else:
  %x2 = sub i32 %a, 1
  br label %merge

merge:
  %x = phi i32 [%x1, %then], [%x2, %else]
```

本專案的 LLVM IR 實作不支援 phi 節點，而是以名稱覆蓋的方式處理（類似非 SSA）。

### 基本區塊 (Basic Block)

線性 sequence 的指令，只有第一個指令可被跳入，只有最後一個指令可跳出（branch 或 return）。

### 控制流圖 (CFG, Control Flow Graph)

由基本區塊組成的有向圖，邊表示控制流轉移。CFG 是最佳化與分析的基礎。

## 本專案編譯器 vs GCC/LLVM

| 特性 | rustc4 | GCC | LLVM (clang) |
|---|---|---|---|
| 前端語言 | Rust 子集 | C/C++/Fortran/... | C/C++/Rust/... |
| 中間表示 | 自訂 LLVM IR 子集 | GIMPLE (三地址碼) | LLVM IR |
| 最佳化 | 無 | 數十道 pass | 數十道 pass |
| 後端 | 無（直接直譯） | 多種架構 | 多種架構 |
| 原始碼 | ~數千行 | 數百萬行 | 數百萬行 |
| 執行方式 | lli4 直譯 | CPU 直接執行 | CPU 直接執行 |
| 用途 | 教學/實驗 | 生產環境 | 生產環境 |

## 前中後端 (Frontend / Middle-end / Backend)

```
原始碼 → [前端] → IR → [中端] → IR → [後端] → 機器碼
         (lex+parse+semantic)  (optimize)   (codegen)
```

本專案的前端是 rustc4，中端被省略，後端被直譯器取代。

## 相關檔案

- `compiler/rustc4/src/lib.rs` — 編譯器入口 (`compile()`)
- `compiler/rustc4/src/lexer.rs` — 詞法分析器
- `compiler/rustc4/src/parser.rs` — 語法分析器
- `compiler/rustc4/src/ast.rs` — AST 定義
- `compiler/rustc4/src/codegen.rs` — IR 生成器
- `compiler/lli4/src/` — IR 直譯器

## 參考資料

- Alfred V. Aho 等人, *Compilers: Principles, Techniques, and Tools* (龍書)
- LLVM 編譯器基礎架構：https://llvm.org/
- rustc 原始碼：https://github.com/rust-lang/rust

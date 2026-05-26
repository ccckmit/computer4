# LLVM IR

## 概述

LLVM IR (Intermediate Representation) 是 LLVM 編譯器基礎架構使用的中間表示式，採用靜態單賦值 (SSA) 形式。本專案的 `rustc4` 將 Rust 原始碼編譯為 `.ir` 格式的 LLVM IR，再由 `lli4` 直譯執行，形成一條完整的編譯器管線。

與標準 LLVM 工具鏈不同，本專案的 LLVM IR 實作是自製的簡化版本，支援 LLVM IR 的核心語意但省略了複雜的最佳化與平台相關功能。

## 本專案的 LLVM IR 實作

### 支援的型別系統

`compiler/lli4/src/ir.rs` 定義了型別系統：

```rust
pub enum LlvmType {
    I8,                        // 8 位元整數
    I32,                       // 32 位元整數
    I1,                        // 1 位元布林
    Void,                      // 無型別（函式回傳）
    Pointer(Box<LlvmType>),    // 指向某型別的指標
    Array(u64, Box<LlvmType>), // 陣列（長度 + 元素型別）
}
```

對比標準 LLVM：標準 LLVM 支援更多型別（浮點數 half/float/double、向量型別、標籤型別、metadata 等），本實作僅聚焦於整數運算。

### 支援的指令

`Instruction` 列舉定義了所有可直譯的指令：

| 指令 | 說明 |
|---|---|
| `Alloca` | 堆疊上分配空間 |
| `Store` | 寫入記憶體 |
| `Load` | 從記憶體讀取 |
| `Add / Sub / Mul / SDiv / SRem` | 整數算術運算 |
| `ICmp` | 整數比較（Eq/Ne/Slt/Sgt/Sle/Sge） |
| `And / Or / Xor` | 位元運算 |
| `Call` | 函式呼叫（可回傳值或無回傳值） |
| `Ret` | 函式回傳 |
| `Br` | 無條件分支 |
| `BrCond` | 條件分支 |
| `GetElementPtr` | GEP 指標運算（陣列索引） |

與標準 LLVM IR 的主要差異：
- 不支援浮點數運算（fadd、fsub 等）
- 不支援 phi 節點（使用 SSA 但透過 greedy 方式處理變數）
- 不支援 load/store 的 align 屬性
- 不支援 volatile、atomic 操作
- GEP 僅支援基本的陣列索引，不支援結構體成員存取

### 指令運算元

```rust
pub enum Operand {
    Int(i64),         // 整數立即值
    Bool(bool),       // 布林立即值
    Local(String),    // 區域變數（SSA 名稱）
    Global(String),   // 全域變數名稱
}
```

### 程式結構

```rust
pub struct Program {
    pub globals: Vec<GlobalVar>,    // 全域變數
    pub functions: Vec<FnDecl>,     // 函式定義
}

pub struct FnDecl {
    pub name: String,
    pub params: Vec<(String, LlvmType)>,
    pub ret_ty: LlvmType,
    pub blocks: Vec<BasicBlock>,    // 基本區塊
}

pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<Instruction>,
}
```

## 編譯器管線運作流程

### rustc4（Rust → LLVM IR）

`compiler/rustc4/` 從 Rust 原始碼編譯出 `.ir` 檔案：

1. **詞法分析：** 將 Rust 原始碼轉換為 token 串流
2. **語法分析：** 解析 Rust 語法結構（函式、變數、表達式、控制流）
3. **IR 生成：** 逐一將 AST 節點轉換為 LLVM IR 指令

`rustc4::compile(source: &str) -> String` 回傳 LLVM IR 文字格式。

支援的 Rust 子集：
- 整數變數宣告與賦值
- 算術與邏輯運算
- if/else 條件分支
- while 迴圈
- 函式定義與呼叫
- 陣列操作（透過 GEP）

### lli4（LLVM IR 直譯器）

`compiler/lli4/` 讀取 `.ir` 檔案並直接執行：

1. **解析：** `parser::parse_ir()` 將文字格式的 IR 轉換為 `Program` 結構
2. **直譯：** `interp::run_program()` 以遞迴直譯方式執行

`lli4::interpret(source: &str) -> String` 接收 IR 文字，回傳程式輸出。

執行模型：
- **堆疊式呼叫框架：** `Vec<Frame>` 作為呼叫堆疊
- **區域變數：** `HashMap<String, i64>` 存放每個 frame 的區域變數
- **記憶體：** `HashMap<u64, i64>` 模擬堆積記憶體
- **輸出緩衝：** 將 `printf`/`putchar` 等輸出附加到 `output: String`

### 完整的編譯→執行流程

```
Rust 原始碼 → rustc4 → .ir 檔案 → lli4 → 輸出結果
```

用法：
```sh
# rustc4 將 Rust 編譯為 .ir
cargo run --bin rustc4 -- input.rs -o output.ir

# lli4 直譯 .ir
cargo run --bin lli4 -- output.ir
```

## LLVM IR 文字格式

本專案使用簡化的 LLVM IR 文字格式（與標準 LLVM 的 `.ll` 格式相似但更簡潔）：

```llvm
@global_var = global i32 42

define i32 @main() {
entry:
    %a = alloca i32
    store i32 10, i32* %a
    %b = load i32, i32* %a
    %c = add i32 %b, 5
    ret i32 %c
}
```

格式特點：
- 全域變數使用 `@` 前綴，區域變數使用 `%` 前綴
- 每個函式由多個基本區塊組成（label 為區塊名稱）
- 每條指令前有可選的 SSA 名稱（`%result = ...`）
- 型別標註置於操作數之前（標準 LLVM 慣例）

## 與標準 LLVM 的比較

| 特性 | 標準 LLVM | 本專案實作 |
|---|---|---|
| 前端 | clang/rustc 等多種 | rustc4（自製 Rust→IR） |
| 型別 | 完整，含浮點/向量/Metadata | 僅整數與指標 |
| 最佳化 | opt（數十道 pass） | 無 |
| 後端 | x86/ARM/RISC-V 等 | 無（直接直譯） |
| 執行 | JIT 或機器碼 | lli4 直譯器 |
| Phi 節點 | 完整支援 | 不支援 |
| 例外處理 | landingpad/invoke | 不支援 |
| 內聯組合語言 | 支援 | 不支援 |

## 在建置此管線時的注意事項

1. rustc4 與 lli4 為獨立 crate，各自有自己的 `Cargo.toml` 與 `target/`
2. `.ir` 檔案為純文字格式，可直接閱覽與除錯
3. lli4 的 Interp 每次重新解析，無 JIT 快取
4. 執行步數無硬性限制，但無限迴圈會導致堆疊溢位
5. printf 等外部函式呼叫必須由 lli4 的直譯器內部處理（非動態鏈結）

## 相關檔案

- `compiler/lli4/src/ir.rs` — IR 資料結構定義
- `compiler/lli4/src/parser.rs` — IR 文字格式解析器
- `compiler/lli4/src/interp.rs` — 直譯器核心（308 行）
- `compiler/rustc4/src/lib.rs` — Rust→IR 編譯器入口
- `compiler/rustc4/src/lex.rs` — 詞法分析器
- `compiler/rustc4/src/parse.rs` — 語法分析器
- `compiler/rustc4/src/codegen.rs` — IR 生成器

## 參考資料

- LLVM Language Reference：https://llvm.org/docs/LangRef.html
- LLVM IR 入門：https://llvm.org/docs/GettingStarted.html
